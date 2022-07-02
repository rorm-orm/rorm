module dorm.migration;

import std.algorithm;
import std.array;
import std.conv;
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
                    "Replaces of migration " ~ lookup[x.dependency].id 
                        ~ " does not exist"
                );
            }
        }

        x.replaces.each!((string y) {
            if ((y in lookup) is null)
            {
            }
        });
    });
    // dfmt on

    // Check that there is never more than one branch
    // TODO

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
    bool testSelfReferencing()
    {
        Migration replaced = Migration(
            "def", true, "0001_replaced", [], ["0001_replaced"], []
        );

        Migration[] test;
        test ~= replaced;
        try
        {
            validateMigrations(test);
            return false;
        }
        catch (MigrationException exc)
        {
            return true;
        }
    }

    assert(testSelfReferencing() == true);
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

    if (!inital)
    {
        validateMigrations(existing);
    }

}
