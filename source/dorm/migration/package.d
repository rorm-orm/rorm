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

    return null;
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
    // Hash is written to each migration file to check if execution is necessary
    auto hash = hashOf(serializedModels.models);

    // dfmt off
    Migration newMigration;

    static Annotation mapAnnotation(SerializedAnnotation annotation)
    {
        return annotation.match!(
            // Annotation flag
            (AnnotationFlag y) => Annotation(y.to!string),
            // ConstructValueRef
            (ConstructValueRef y) => Annotation("ConstructValue", AnnotationType(y.id)),
            // ValidatorRef
            (ValidatorRef y) => Annotation("Validator", AnnotationType(y.id)),
            // maxLength
            (maxLength y) => Annotation("MaxLength", AnnotationType(y.maxLength)),
            // PossibleDefaultValueTs
            oneOf!((allPossibleValues) {
                return Annotation("DefaultValue", AnnotationType(allPossibleValues.value));
            }, PossibleDefaultValueTs),
            // Choices
            (Choices y) => Annotation("Choices", AnnotationType(y.choices.map!(
                z => AnnotationType(z.to!string)
            ).array)),
            // index
            (index y) {
                auto table = cast(AnnotationType[string])[
                    "Priority": AnnotationType(y._priority.priority),
                ];
                if (y._composite.name.length > 0) {
                    table["Name"] = y._composite.name;
                }
                
                return Annotation("Index", AnnotationType(table));
            }
        );
    }

    newMigration.operations = serializedModels.models.map!(x => OperationType(
        CreateModelOperation(
            x.name,
            x.fields.map!(
                y => Field(
                    y.name,
                    y.type,
                    y.annotations.map!(mapAnnotation).array 
                        ~ (y.nullable ? [] : [Annotation("notNull")])
                )
            ).array
        )
    )).array;
    // dfmt on

    auto outFile = File(buildPath(migrationDirectory, "0002_abc.toml"), "w");
    outFile.writeln(serializeMigration(newMigration));

}
