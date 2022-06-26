module dorm.migration.parser;

import core.exception;

import std.algorithm;
import std.conv;
import std.file;
import std.stdio;

import dorm.migration.declaration;
import dorm.exceptions;

import toml;

/** 
 * Helper function to check if a parameter exists in the given table
 * and is of the desired type.
 * 
 * Params:
 *   keyName = Key to search for
 *   table   = Reference to map of string : TOMLValue to search in
 *   type    = Desired type
 *   path    = Path to migration file, used for exceptions
 *
 * Throws: dorm.exceptions.MigrationException if key was not
 *      found or has the wrong type
 */
private void checkValueExists(
    string keyName, ref TOMLValue[string] table, TOML_TYPE type, string path
)
{
    if ((keyName in table) is null)
    {
        throw new MigrationException(
            "missing key " ~ keyName ~ " of type "
                ~ type ~ " in migration file " ~ path
        );
    }

    if (table[keyName].type != type)
    {
        throw new MigrationException(
            "key " ~ keyName ~ " is of the wrong type. Should be of type "
                ~ to!string(
                    type) ~ ". Is " ~ to!string(table[keyName].type)
                ~ ". Migration file " ~ path
        );

    }
}

/** 
 * Helper function to parse migration files.
 * 
 * Params:
 *   path = Path to the file that should be parsed
 *
 * Returns: 
 *
 * Throws: dorm.exceptions.MigrationException
 */
Migration parseFile(string path)
{
    void[] data;
    try
    {
        data = read(path);
    }
    catch (FileException exc)
    {
        throw new MigrationException(
            "could not read migration file: " ~ path, exc
        );
    }

    try
    {
        auto doc = parseTOML(cast(string) data);

        Migration migration;

        checkValueExists("Migration", doc.table, TOML_TYPE.TABLE, path);
        TOMLValue migrationSection = doc.table["Migration"];

        checkValueExists("Hash", migrationSection.table, TOML_TYPE.STRING, path);
        migration.hash = migrationSection.table["Hash"].str;

        if (("Initial" in migrationSection.table) is null)
        {
            throw new MigrationException(
                "missing key Initial of type BOOL in migration file " ~ path
            );
        }
        if (migrationSection.table["Initial"].type == TOML_TYPE.TRUE)
        {
            migration.initial = true;
        }
        else if (migrationSection.table["Initial"].type == TOML_TYPE.FALSE)
        {
            migration.initial = false;
        }
        else
        {
            throw new MigrationException(
                "key Initial has the wrong type. Should be of type BOOL. Is " ~ to!string(
                    migrationSection.table["Initial"].type) ~ ". Migration file: " ~ path
            );
        }

        checkValueExists("Dependencies", migrationSection.table, TOML_TYPE.ARRAY, path);
        TOMLValue[] dependencies = migrationSection.table["Dependencies"].array;
        dependencies.each!((TOMLValue x) {
            if (x.type != TOML_TYPE.STRING)
            {
                // dfmt off
                throw new MigrationException(
                    "type of Migration.Dependencies member is wrong. Expected: "
                    ~ to!string(TOML_TYPE.STRING) ~ ". Found "
                    ~ to!string(x.type) ~ "Migration file: " ~ path
                );
                //dfmt on
            }
            migration.dependencies ~= x.str;
        });

        checkValueExists("Replaces", migrationSection.table, TOML_TYPE.ARRAY, path);
        TOMLValue[] replaces = migrationSection.table["Replaces"].array;
        replaces.each!((TOMLValue x) {
            if (x.type != TOML_TYPE.STRING)
            {
                // dfmt off
                throw new MigrationException(
                    "type of Migration.Replaces member is wrong. Expected: "
                    ~ to!string(TOML_TYPE.STRING) ~ ". Found "
                    ~ to!string(x.type) ~ "Migration file: " ~ path
                );
                //dfmt on
            }
            migration.replaces ~= x.str;
        });

        checkValueExists("Operations", migrationSection.table, TOML_TYPE.ARRAY, path);
        TOMLValue[] operations = migrationSection.table["Operations"].array;

        // TODO: Implement operations

        return migration;
    }

    catch (TOMLParserException exc)
    {
        throw new MigrationException(
            "could not parse migration file " ~ path, exc
        );
    }
}

unittest
{
    import std.path;

    string test = `
    [Migration]
    Hash = "1203019591923"
    Initial = true
    Dependencies = ["01", "02"]
    Replaces = ["01_old"]

    [[Migration.Operations]]
    `;

    auto fh = File(buildPath(tempDir(), "dormmigrationtest.toml"), "w");
    fh.writeln(test);
    fh.close();

    auto correct = Migration(
        "1203019591923", true, "3", ["01", "02"], ["01_old"], []
    );
    auto toTest = parseFile(fh.name());
    assert(correct.dependencies == toTest.dependencies);
    assert(correct.operations == toTest.operations);
    assert(correct.replaces == toTest.replaces);
    assert(correct.initial == toTest.initial);
    assert(correct.hash == toTest.hash);

    remove(fh.name());
}
