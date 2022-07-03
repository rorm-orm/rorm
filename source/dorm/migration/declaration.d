module dorm.migration.declaration;

import std.datetime : Date, DateTime, TimeOfDay, SysTime;
import std.sumtype;

import dorm.declarative;

import toml;
import toml.serialize;

/** 
 * The migration type can be determined by using this
 */
alias OperationType = SumType!(CreateModelOperation, DeleteModelOperation);

alias DBType = ModelFormat.Field.DBType;

alias AnnotationType = SumType!(
    ubyte[], double, string, long, Date, DateTime,
    TimeOfDay, SysTime, This[], This[string]
);

const string[] annotationsWithValue = [
    "Choices", "ConstructValue", "DefaultValue",
    "Index", "MaxLength", "Validator"
];

const string[] annotationsWithoutValue = [
    "NotNull", "AutoUpdateTime", "AutoCreateTime", "PrimaryKey", "Unique"
];

/** 
 * Representation of an annotation
 */
struct Annotation
{
    /// Type of the annotation
    @tomlName("Type")
    string type;

    @tomlName("Value")
    AnnotationType value;
}

/** 
 * Representation of a database column
 */
struct Field
{
    /// Name of the field
    @tomlName("Name")
    string name;

    /// Type of the field
    @tomlName("Type")
    DBType type;

    /// List of serialized annotations
    @tomlName("Annotations")
    Annotation[] annotations;
}

/** 
 * Operation that represents the creation of a model
 */
struct CreateModelOperation
{
    /// Name of the model to execute the operation on
    @tomlName("Name")
    string name;

    /// Fields of the model
    @tomlName("Fields")
    Field[] fields;
}

/**
 * Operation that represents the deletion of a model
 */
struct DeleteModelOperation
{
    /// Name of the model that should be deleted
    @tomlName("Name")
    string name;
}

/** 
 * The base struct for every migration file
 */
struct Migration
{
    /// Hash of the migration
    @tomlName("Hash")
    long hash;

    /// Marks the migration initial state
    @tomlName("Initial")
    bool initial;

    /// ID of the mirgation, derived from filename
    string id;

    /// Migration this migration depends on
    @tomlName("Dependency")
    string dependency;

    /// List of migrations this migration replaces
    @tomlName("Replaces")
    string[] replaces;

    /// The operations to execute
    @tomlName("Operations")
    OperationType[] operations;
}
