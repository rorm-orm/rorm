/**
 * This whole package is used for the declarative model descriptions. The
 * declarative descriptions are automatically generated from the D source code
 * and are used for the diff process for the migrations generator.
 *
 * The conversion from D classes/structs + UDAs into the declarative format
 * described in this module is done inside the $(REF conversion, dorm,declarative)
 * module.
 */
module dorm.declarative;

import dorm.annotations;
import dorm.model;

import std.sumtype;
import std.typecons : tuple;

/**
 * This is the root of a described models module. It contains a list of models
 * as defined in the D source file.
 *
 * The `validators` and `valueConstructors` maps contain the global functions
 * defined in the $(REF defaultValue, dorm,annotations) and $(REF validator,
 * dorm,annotations) UDAs.
 */
struct SerializedModels
{
	/// List of all the models defined in the full module file.
	ModelFormat[] models;

	/** 
	 * Global (per-SerializedModels instance) list of validator and defaultValue
	 * functions.
	 *
	 * The key is an opaque integer that is referenced through the $(LREF
	 * ConstructValueRef) and $(LREF ValidatorRef) annotations inside the
	 * ModelFormat.Field.annotations field. These IDs may or may not stay the
	 * same across builds and there is no meaning inside them.
	 *
	 * The value (function pointer) is a global function that is compiled into
	 * the executable through the call of $(REF processModelsToDeclarations,
	 * dorm,declarative) generating the `SerializedModels` definition. Manually
	 * calling this function is not required, use the (TODO: need mixin) mixin
	 * instead.
	 *
	 * The functions take in a Model (class) instance, assert it is the correct
	 * model class type that it was registered with.
	 *
	 * The `validator` function calls the UDA lambda with the field as argument
	 * and returns its return value, with the code assuming it is a boolean.
	 * (a compiler error will occur if it cannot implicitly convert to `bool`)
	 *
	 * The `valueConstructor` function calls the UDA lambda without argument and
	 * sets the annotated field value inside the containing Model instance to
	 * its return value, with the code assuming it can simply assign it.
	 * (a compiler error will occur if it cannot implicitly convert to the
	 * annotated property type)
	 */
	bool function(Model)[int] validators;
	/// ditto
	void function(Model)[int] valueConstructors;
}

/** 
 * Describes a single Model class (Database Table) in a generic format that is
 * only later used by the drivers to actually convert to SQL statements.
 */
struct ModelFormat
{
	/** 
	 * Describes a field inside the Model class, which corresponds to a column
	 * inside the actual database table later. It's using a generic format that
	 * is only later used by the drivers to actually convert to SQL statements.
	 */
	struct Field
	{
		/// List of different (generic) database column types.
		enum DBType
		{
			varchar, /// inferred from `string`
			varbinary, /// inferred from `ubyte[]`
			int8, /// inferred from `byte`
			int16, /// inferred from `short`
			int32, /// inferred from `int`
			int64, /// inferred from `long`
			uint8, /// inferred from `ubyte`
			uint16, /// inferred from `ushort`
			uint32, /// inferred from `uint`
			uint64, /// inferred from `ulong`
			floatNumber, /// inferred from `float`
			doubleNumber, /// inferred from `double`
			boolean, /// inferred from `bool`
			date, /// inferred from `std.datetime : Date`
			datetime, /// inferred from `std.datetime : DateTime`
			timestamp, /// inferred from `std.datetime : SysTime`, `@AutoCreateTime ulong`, `@AutoUpdateTime ulong`, `@timestamp ulong`
			time, /// inferred from `std.datetime : TimeOfDay`
			choices, /// inferred from `@choices string`, `enum T : string`
			set, /// inferred from `BitFlags!enum`
		}

		/// The exact name of the column later used in the DB, not neccessarily
		/// corresponding to the D field name anymore.
		string name;
		/// The generic column type that is later translated to a concrete SQL
		/// type by a driver.
		DBType type;
		/// `true` if this column can be NULL in SQL, otherwise `false`.
		/// Using the D $(REF Nullable, std,typecons) type will automatically
		/// set this to true. This is also null when embedding other Models as
		/// references.
		bool nullable;
		/// List of different annotations defined in the source code, converted
		/// to a serializable format and also all implicit annotations such as
		/// `Choices` for enums.
		SerializedAnnotation[] annotations;
		/// For debugging purposes this is the D source code location where this
		/// field is defined from. This can be used in error messages.
		SourceLocation definedAt;

		size_t toHash() const @nogc @safe pure nothrow
		{
			return hashOf(tuple(name, type, nullable, annotations));
		}
	}

	/// The exact name of the table later used in the DB, not neccessarily
	/// corresponding to the D class name anymore.
	string name;
	/// For debugging purposes this is the D source code location where this
	/// field is defined from. This can be used in error messages.
	SourceLocation definedAt;
	/// List of fields, such as defined in the D source code, recursively
	/// including all fields from all inherited classes. This maps to the actual
	/// SQL columns later when it is generated into an SQL create statement by
	/// the actual driver implementation.
	Field[] fields;

	size_t toHash() const @nogc @safe pure nothrow
	{
		return hashOf(tuple(name, fields));
	}
}

/**
 * The source location where something is defined in D code.
 *
 * The implementation uses [__traits(getLocation)](https://dlang.org/spec/traits.html#getLocation)
 */
struct SourceLocation
{
	/// The D filename, assumed to be of the same format as [__FILE__](https://dlang.org/spec/expression.html#specialkeywords).
	string sourceFile;
	/// The 1-based line number and column number where the symbol is defined.
	int sourceLine, sourceColumn;
}

/**
 * This enum contains all no-argument flags that can be added as annotation to
 * the fields. It's part of the $(LREF SerializedAnnotation) SumType.
 */
enum AnnotationFlag
{
	/// corresponds to the $(REF autoCreateTime, dorm,annotations) UDA.
	AutoCreateTime,
	/// corresponds to the $(REF autoUpdateTime, dorm,annotations) UDA.
	AutoUpdateTime,
	/// corresponds to the $(REF primaryKey, dorm,annotations) UDA.
	PrimaryKey,
	/// corresponds to the $(REF unique, dorm,annotations) UDA.
	Unique
}

/**
 * SumType combining all the different annotations (UDAs) that can be added to
 * a model field, in a serializable format. (e.g. the lambdas are moved into a
 * helper field in the model description and these annotations only contain an
 * integer to reference it)
 */
alias SerializedAnnotation = SumType!(
	AnnotationFlag, ConstructValueRef, ValidatorRef,
	maxLength, PossibleDefaultValueTs, Choices, index
);

/**
 * Corresponds to the $(REF constructValue, dorm,annotations) UDA, but the
 * function being moved out this value into the $(LREF SerializedModels) struct.
 */
struct ConstructValueRef
{
	/// opaque id to use as index inside SerializedModels.valueConstructors
	long id;
}

/**
 * Corresponds to the $(REF constructValue, dorm,annotations) UDA, but the
 * function being moved out this value into the $(LREF SerializedModels) struct.
 */
struct ValidatorRef
{
	/// opaque id to use as index inside SerializedModels.validators
	long id;
}
