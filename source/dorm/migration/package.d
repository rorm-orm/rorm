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

/** 
 * Configuration for migrations
 */
struct MigrationConfig
{
    string migrationDirectory;
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
private void checkMigrationsStructure(ref MigrationConfig conf)
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
    checkMigrationsStructure(conf);

    auto entries = dirEntries(
        conf.migrationDirectory,
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
 *   serializedModels = Helper model that contains the parsed models.
 *   conf = Configuration of the migration creation / parsing process
 *
 * Throws: 
 *   dorm.exceptions.MigrationException
 */
void makeMigrations(SerializedModels serializedModels, MigrationConfig conf)
{
    Migration[] existing = getExistingMigrations(conf);
    //auto outFile = File(buildPath(migrationDirectory, "0002_abc.toml"), "w");
    //outFile.writeln(serializeMigration(newMigration));

}
