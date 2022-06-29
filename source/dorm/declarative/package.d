module dorm.declarative;

import dorm.annotations;
import dorm.model;

import std.sumtype;
import std.typecons : tuple;

struct SerializedModels
{
	ModelFormat[] models;

	bool function(Model)[int] validators;
	void function(Model)[int] valueConstructors;
}

struct ModelFormat
{
	struct Field
	{
		enum DBType
		{
			varchar = "varchar", // string
			varbinary = "varbinary", // ubyte[]
			int8 = "int8", // byte
			int16 = "int16", // short
			int32 = "int32", // int
			int64 = "int64", // long
			uint8 = "uint8", // ubyte
			uint16 = "uint16", // ushort
			uint32 = "uint32", // uint
			uint64 = "uint64", // ulong
			floatNumber = "floatNumber", // float
			doubleNumber = "doubleNumber", // double
			boolean = "boolean", // bool
			date = "date", // std.datetime : Date
			datetime = "datetime", // std.datetime : DateTime
			timestamp = "timestamp", // std.datetime : SysTime, @autoCreateTime ulong, @autoUpdateTime ulong, @timestamp ulong
			time = "time", // std.datetime : TimeOfDay
			choices = "choices", // @choices string, enum:string
			set = "set", // BitFlags!enum
		}

		string name;
		DBType type;
		bool nullable;
		SerializedAnnotation[] annotations;
		SourceLocation definedAt;

		size_t toHash() const @nogc @safe pure nothrow
		{
			return hashOf(tuple(name, type, nullable, annotations));
		}
	}

	string name;
	SourceLocation definedAt;
	Field[] fields;

	size_t toHash() const @nogc @safe pure nothrow
	{
		return hashOf(tuple(name, fields));
	}
}

/// the source location where something is defined in D code
struct SourceLocation
{
	string sourceFile;
	int sourceLine, sourceColumn;
}

enum AnnotationFlag
{
	autoCreateTime = "AutoCreateTime",
	autoUpdateTime = "AutoUpdateTime",
	primaryKey = "PrimaryKey",
	unique = "Unique"
}

alias SerializedAnnotation = SumType!(
	AnnotationFlag, ConstructValueRef, ValidatorRef,
	maxLength, PossibleDefaultValueTs, Choices, index
);

struct ConstructValueRef
{
	long id;
}

struct ValidatorRef
{
	long id;
}
