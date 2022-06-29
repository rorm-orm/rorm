module dorm.migration;

import std.algorithm;
import std.array;
import std.conv;
import std.file;
import std.path;
import std.range;
import std.sumtype;
import std.stdio;

import dorm.annotations;
import dorm.declarative;
import dorm.exceptions;
import dorm.migration.declaration;
import dorm.migration.parser;

import toml;

const migrationDirectory = "migrations";

private void checkMigrationsStructure()
{
    if (!exists(migrationDirectory))
    {
        try
        {
            mkdir(migrationDirectory);
        }
        catch (FileException exc)
        {
            throw new MigrationException(
                "Could not create " ~ migrationDirectory ~ " directory",
                exc
            );
        }
    }
    else
    {
        if (!isDir(migrationDirectory))
        {
            throw new MigrationException(
                "Migration directory " ~ migrationDirectory ~ " is a file"
            );
        }
    }
}

/** 
 * Use this method to retrieve the existing migrations
 */
Migration[] getExistingMigrations()
{
    checkMigrationsStructure();

    auto entries = dirEntries(
        migrationDirectory,
        "[0123456789][0123456789][0123456789][0123456789]_?*.toml",
        SpanMode.shallow,
        false
    ).filter!(x => x.isFile())
        .array
        .schwartzSort!(x => baseName(x.name)[0 .. 4].to!ushort);

    return entries.map!(x => parseFile(x.name())).array;
}

/** 
 * Intended to get called after conversion.
 *
 * Params:
 *   SerializedModels = Helper model that contains the parsed models.
 *
 * Throws: 
 *   dorm.exceptions.MigrationException
 */
void makeMigrations(SerializedModels serializedModels)
{
    Migration[] existing = getExistingMigrations();
    //auto outFile = File(buildPath(migrationDirectory, "0002_abc.toml"), "w");
    //outFile.writeln(serializeMigration(newMigration));

}
