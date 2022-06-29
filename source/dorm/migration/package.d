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
}
