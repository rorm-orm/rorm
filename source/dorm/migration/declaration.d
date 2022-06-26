module dorm.migration.declaration;

import std.sumtype;

import dorm.declarative;

/** 
 * Type of the database operation to execute
 */
enum OperationType
{
    CreateModel = "CreateModel",
    RemoveModel = "RemoveModel",
    AlterModel = "AlterModel",
    AddField = "AddField",
    RemoveField = "RemoveField",
    AlterField = "AlterField",
    RenameField = "RenameField",
}

/** 
 * 
 */
struct Annotation
{
    /// Name of the annotation
    string name;

}

/** 
 * Representation of a database column
 */
struct Field
{
    alias DBType = ModelFormat.Field.DBType;

    /// Name of the field
    string name;

    /// Type of the field
    DBType type;

    // Nullable flag
    bool nullable;

    /// List of serialized annotations
    SerializedAnnotation[] annotations;
}

/** 
 * The operation
 */
struct Operation
{
    /// Name of the model
    string modelName;

    /// OperationType
    OperationType type;

    /// List of fields
    Field[] fields;
}

/** 
 * The base struct for every migration file
 */
struct Migration
{
    /// Hash of the migration
    string hash;

    /// Marks the migration initial state
    bool initial;

    /// ID of the mirgation, derived from filename
    string id;

    /// List of migrations this migration depends on
    string[] dependencies;

    /// List of migrations this migration replaces
    string[] replaces;

    /// The operations to execute
    Operation[] operations;
}
