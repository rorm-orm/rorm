module dorm.migration;

import std.algorithm;
import std.array;
import std.conv;
import std.exception;
import std.file;
import std.path;
import std.range;
import std.regex;
import std.sumtype;
import std.stdio;

import dorm.annotations;
import dorm.declarative;
import dorm.exceptions;
import dorm.migration.declaration;
import dorm.migration.parser;

import toml;

/** 
 * Configuration for migrations
 */
struct MigrationConfig
{
    /// Directory where migration files can be found
    string migrationDirectory;

    /// If set to true, logging is disabled
    bool loggingDisabled;
}

/** 
 * Helper method to check the migration directory structure.
 * 
 * If the directory specified in conf.migrationDirectory does not exists,
 * it will be created.
 *
 * Params: 
 *   conf = Reference to the MigrationConfig
 *
 * Throws: MigrationException if the migrationDirectory could not be created
 */
private void checkDirectoryStructure(ref MigrationConfig conf)
{
    if (!exists(conf.migrationDirectory))
    {
        try
        {
            mkdir(conf.migrationDirectory);
        }
        catch (FileException exc)
        {
            throw new MigrationException(
                "Could not create " ~ conf.migrationDirectory ~ " directory",
                exc
            );
        }
    }
    else
    {
        if (!isDir(conf.migrationDirectory))
        {
            throw new MigrationException(
                "Migration directory " ~ conf.migrationDirectory ~ " is a file"
            );
        }
    }
}

/** 
 * Use this method to retrieve the existing migrations
 *
 * Params: 
 *   conf = Reference to MigrationConfig
 *
 * Throws: MigrationException in various cases
 */
Migration[] getExistingMigrations(ref MigrationConfig conf)
{
    checkDirectoryStructure(conf);

    auto entries = dirEntries(
        conf.migrationDirectory,
        "?*.toml",
        SpanMode.shallow,
        false
    ).filter!(x => x.isFile())
        .filter!(
            (DirEntry x) {
            if (!baseName(x.name).matchFirst(`^[0-9]{4}_\w+\.toml$`))
            {
                if (!conf.loggingDisabled)
                {
                    stderr.writeln(
                        "WARNING: Ignoring " ~ baseName(x.name)
                        ~ " as migration file."
                    );
                }
                return false;
            }
            return true;
        }
        )
        .array
        .schwartzSort!(x => baseName(x.name)[0 .. 4].to!ushort);

    return entries.map!(x => parseFile(x.name())).array;
}

/** 
 * Validate the given migrations in terms of their set
 * dependencies, replaces and initial values
 *
 * Params:
 *   existing = Reference to an array of Migrations
 *
 * Throws:
 *   dorm.exceptions.MigrationException
 */
void validateMigrations(ref Migration[] existing)
{
    // If there are no migrations, there's nothing to do
    if (existing.length == 0)
        return;

    Migration[string] lookup;

    // Build lookup table
    foreach (migration; existing)
    {
        // If an id is existing more than once, throw
        if ((migration.id in lookup) !is null)
        {
            throw new MigrationException(
                "Migration id found multiple times: " ~ migration.id
            );
        }

        lookup[migration.id] = migration;
    }

    // dfmt off
    // Check that all dependencies && replaces exist
    existing.each!((Migration x) {

        if (x.dependency != "")
        {
            if ((x.dependency in lookup) is null)
            {
                throw new MigrationException(
                    "Dependency of migration " ~ lookup[x.dependency].id 
                        ~ " does not exist"
                );
            }
        }

        x.replaces.each!((string y) {
            if ((y in lookup) is null)
            {
                throw new MigrationException(
                    "Replaces of migration " ~ lookup[y].id 
                        ~ " does not exist"
                );
            }
        });
    });
    // dfmt on

    // Check that there is never more than one branch
    string[][string] depList;
    existing.each!((Migration x) {
        if (x.dependency != "")
        {
            depList[x.dependency] ~= x.id;
        }
    });

    string[][string] faulty;
    depList.byKeyValue.each!((x) {
        if (x.value.length > 1)
        {
            auto remaining = x.value.dup;
            x.value.each!(
                y => lookup[y].replaces.each!(
                z => remaining = remaining.remove!(a => a == z)
            )
            );

            if (remaining.length > 1)
            {
                faulty[x.key] ~= remaining;
            }

        }
    });

    if (faulty.length > 0)
    {
        throw new MigrationException(
            "Following migrations have the same dependencies (= branching): "
                ~ faulty.byKeyValue.map!(
                    x => x.value.join(", ") ~ " referencing "
                    ~ x.key ~ " as dependency"
                ).join("; ")
        );
    }

    // Check if migration that has initial = false has no dependencies
    existing.each!((Migration x) {
        if (!x.initial && x.dependency is null)
        {
            throw new MigrationException(
                "No dependencies found on migration with initial = false: "
                ~ x.id
            );
        }
    });

    // Check if replaces or dependencies create a loop
    string[] visited;

    void checkLoop(Migration m)
    {
        // If current id is in visited, that's a loop
        if (visited.canFind(m.id))
        {
            throw new MigrationException(
                "Detected loop in replaces: " ~ join(visited, " ")
            );
        }

        visited ~= m.id;

        m.replaces.each!(x => checkLoop(lookup[x]));

    }

    auto replaceMigrations = existing.filter!(x => x.replaces.length > 0);
    foreach (key; replaceMigrations)
    {
        visited = [];
        checkLoop(key);
    }

    auto dependencyMigrations = existing.filter!(x => x.dependency != "");
    foreach (key; dependencyMigrations)
    {
        visited = [];
        checkLoop(key);
    }

    // Check if none or more than one migration has inital set to true
    auto initialMigrations = existing.filter!(x => x.initial);

    // If any migrations with initial = true, throw
    initialMigrations.each!((Migration x) {
        if (x.dependency != "")
        {
            throw new MigrationException(
                "Migration with initial = true cannot have dependencies: "
                ~ x.id
            );
        }
    });

    if (initialMigrations.count() == 0)
    {
        throw new MigrationException("Couldn't find an inital migration.");
    }
    else if (initialMigrations.count() > 1)
    {
        Migration[string] toCheck;
        initialMigrations.each!((Migration x) { toCheck[x.id] = x; });

        string initialMigration;

        void checkMigration(Migration m)
        {
            // Check if migration replaces another migration
            if (m.replaces.length > 0)
            {
                auto replaces = m.replaces.filter!(x => lookup[x].initial);

                // dfmt off
                // If no migration has initial = true, throw
                if (replaces.count() == 0)
                {
                    throw new MigrationException(
                        "Migration with inital = true replaces other migrations"
                            ~ " without inital = true: " ~ m.id
                    );
                }
                // If more than one migrations have initial = true, throw
                else if (replaces.count() > 1)
                {
                    throw new MigrationException(
                        "Migration with initial = true replaces other migrations"
                            ~ " where multiple have initial = true set: " ~ m.id
                    );
                }

                // dfmt on

                foreach (mID; m.replaces)
                {
                    if (lookup[mID].initial)
                    {
                        checkMigration(lookup[mID]);
                    }
                }
            }
            else
            {
                if (initialMigration != "")
                {
                    throw new MigrationException(
                        "There are multiple independent initial migrations: "
                            ~ initialMigration ~ " : " ~ m.id
                    );
                }
                initialMigration = m.id;
            }

            // Remove Migration from toCheck list
            toCheck.remove(m.id);
        }

        while (toCheck.length != 0)
        {
            Migration migration = toCheck.byKeyValue.front.value;
            checkMigration(migration);
        }

    }
}

unittest
{
    Migration replaced = Migration(
        "def", true, "0001_replaced", "", ["0001_replaced"], []
    );

    Migration[] test;
    test ~= replaced;
    assertThrown!MigrationException(validateMigrations(test));
}

/** 
 * Removes migrations that have been replaced.
 * validateMigrations must be executed first.
 *
 * Params:
 *   existing = Checked list of migrations
 *
 * Returns: Cleaned list of migrations
 */
Migration[] cleanMigrations(ref Migration[] existing)
{
    Migration[string] cleaned;
    existing.each!((Migration x) { cleaned[x.id] = x; });

    auto replaceMigrations = existing.filter!(x => x.replaces.length > 0);
    replaceMigrations.each!(x => x.replaces.each!(y => cleaned.remove(y)));

    return cleaned.byValue.array;
}

/** 
 * Helper function to order migrations. It also removes migrations
 * that are replaced.
 *
 * Params:
 *   existing = Reference to list of migrations
 *
 * Returns: Ordered list of migrations
 *
 * Throws: dorm.exceptions.MigrationException
 */
Migration[] orderMigrations(ref Migration[] existing)
{
    // Validate migrations first
    validateMigrations(existing);
    Migration[] cleaned = cleanMigrations(existing);

    Migration search;

    Migration[] ordered;

    auto first = cleaned.filter!(x => x.initial);
    if (first.empty)
    {
        return ordered;
    }
    Migration head = first.front;
    ordered ~= head;
    auto sortedRange = cleaned.sort!((a, b) => a.dependency < b.dependency);

    while (true)
    {
        search.dependency = head.id;
        auto eq = sortedRange.equalRange(search);
        if (eq.empty)
        {
            break;
        }
        assert(eq.length == 1);
        ordered ~= eq.front;
        head = eq.front;
    }

    return ordered;
}

unittest
{
    auto elem1 = Migration(
        "", true, "0001_abc", "", [], [
            OperationType(CreateModelOperation())
        ]
    );
    auto elem2 = Migration(
        "", false, "0002_abc", "0001_abc", [], [
            OperationType(CreateModelOperation())
        ]
    );
    auto elem3 = Migration(
        "", false, "0003_abc", "0002_abc", [], [
            OperationType(CreateModelOperation())
        ]
    );
    auto elem4 = Migration(
        "", false, "0002_new", "0001_abc", ["0002_abc", "0003_abc"], [
            OperationType(CreateModelOperation())
        ]
    );

    Migration[] toTest;
    toTest ~= elem3;
    toTest ~= elem1;
    toTest ~= elem2;

    assert(orderMigrations(toTest) == [elem1, elem2, elem3]);

    toTest ~= elem4;

    assert(orderMigrations(toTest) == [elem1, elem4]);
}

/** 
 * Intended to get called after conversion.
 *
 * Params:
 *   serializedModels = Helper model that contains the parsed models.
 *   conf = Configuration of the migration creation / parsing process
 *
 * Throws: 
 *   dorm.exceptions.MigrationException
 */
void makeMigrations(SerializedModels serializedModels, MigrationConfig conf)
{
    Migration[] existing = getExistingMigrations(conf);

    // If there are no existing migrations
    // the new migration will be the inital one
    bool inital = existing.length == 0;

    Migration[] ordered = orderMigrations(existing);
    writeln(ordered);

}
