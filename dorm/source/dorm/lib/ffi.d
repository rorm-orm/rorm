module dorm.lib.ffi;

extern(C):

struct FFIArray(T)
{
	T* content;
	size_t size;

	T[] data() nothrow pure @nogc return
	{
		return content[0 .. size];
	}

	static FFIArray fromData(return T[] data) nothrow pure @nogc
	{
		return FFIArray(data.ptr, data.length);
	}

	static FFIArray fromData(size_t n)(return ref T[n] data) nothrow pure @nogc
	{
		return FFIArray(data.ptr, data.length);
	}
}

alias FFIString = FFIArray!(const(char));

FFIString ffi(string s) { return FFIString.fromData(s); }
FFIArray!T ffi(T)(T[] s) { return FFIArray!T.fromData(s); }
FFIArray!T ffi(T, size_t)(ref T[n] s) { return FFIArray!T.fromData(s); }

enum DBBackend
{
	SQLite,
	Postgres,
	MySQL
}

struct DBConnectOptions
{
	DBBackend backend;
	FFIString name;
	FFIString host;
	ushort port;
	FFIString user;
	FFIString password;
	uint minConnections;
	uint maxConnections;
}

alias DBHandle = void*;

alias DBConnectCallback = extern(C) void function(void* data, FFIResult!DBHandle result) nothrow;
void rorm_db_connect(DBConnectOptions options, DBConnectCallback callback, void* data);

void rorm_shutdown(ulong timeoutMsecs);

// hypothetical:

struct FFIResult(T)
{
	T raw_result;
	FFIString error;

	T expect() return
	{
		if (error.size)
			throw new Exception(error.data.idup);

		return raw_result;
	}
}

void rorm_db_discconnect(DBHandle handle);

struct Condition
{
	enum Type
	{
		Conjunction,
		Disjunction,
		UnaryCondition,
		BinaryCondition,
		TernaryCondition,
		Value
	}
	Type type;

	union
	{
		FFIArray!Condition conjunction;
		FFIArray!Condition disjunction;
		UnaryCondition* unaryCondition;
		BinaryCondition* binaryCondition;
		TernaryCondition* ternaryCondition;
		ConditionValue value;
	}
}

struct UnaryCondition
{
	enum Type
	{
		IsNull,
		IsNotNull,
		Exists,
		NotExists
	}
	Type type;

	Condition condition;
}

struct BinaryCondition
{
	enum Type
	{
		Equals,
		NotEquals,
		Greater,
		GreaterOrEquals,
		Less,
		LessOrEquals,
		Like,
		NotLike,
		Regexp,
		NotRegexp,
		In,
		NotIn
	}
	Type type;

	Condition lhs;
	Condition rhs;
}

struct TernaryCondition
{
	enum Type
	{
		Between,
		NotBetween
	}
	Type type;

	Condition first;
	Condition second;
	Condition third;
}

struct ConditionValue
{
	enum Type
	{
		Identifier,
		String,
		I64,
		I32,
		I16,
		Bool,
		F64,
		F32,
		Null
	}
	Type type;

	union
	{
		FFIString identifier;
		FFIString string;
		FFIString i64;
		FFIString i32;
		FFIString i16;
		FFIString boolean;
		FFIString f64;
		FFIString f32;
	}
}

alias DBRowHandle = void*;
alias DBStreamHandle = void*;

DBStreamHandle rorm_db_query_stream(DBHandle handle, FFIString model, FFIArray!FFIString columns);

/// Returns true if the stream pointed to by the handle is invalid or empty.
bool rorm_stream_empty(DBStreamHandle handle);

alias StreamRowCallback = extern(C) void function(void* data, scope FFIResult!DBRowHandle row);

/// Returns the current item pointed to by the stream and advances it. If
/// already past the end or on an invalid stream, an error is passed in in the
/// result. The callback is called synchronously 
void rorm_stream_next(DBStreamHandle handle, StreamRowCallback callback, void* data);

/**
Params:
	handle = row handle to read from
	columnIndex = the column index (array index from the `columns` parameter in
		the corresponding $(LREF rorm_db_query_stream) call)

Returns:
	The extracted value from the row at the given column index. FFIArray and
	FFIString values must be copied if using them outside the lifetime of the
	row.
*/
byte rorm_row_get_data_int8(DBRowHandle handle, size_t columnIndex);
short rorm_row_get_data_int16(DBRowHandle handle, size_t columnIndex); /// ditto
int rorm_row_get_data_int32(DBRowHandle handle, size_t columnIndex); /// ditto
long rorm_row_get_data_int64(DBRowHandle handle, size_t columnIndex); /// ditto
ubyte rorm_row_get_data_uint8(DBRowHandle handle, size_t columnIndex); /// ditto
ushort rorm_row_get_data_uint16(DBRowHandle handle, size_t columnIndex); /// ditto
uint rorm_row_get_data_uint32(DBRowHandle handle, size_t columnIndex); /// ditto
ulong rorm_row_get_data_uint64(DBRowHandle handle, size_t columnIndex); /// ditto
float rorm_row_get_data_float_number(DBRowHandle handle, size_t columnIndex); /// ditto
double rorm_row_get_data_double_number(DBRowHandle handle, size_t columnIndex); /// ditto
bool rorm_row_get_data_boolean(DBRowHandle handle, size_t columnIndex); /// ditto
FFIString rorm_row_get_data_varchar(DBRowHandle handle, size_t columnIndex); /// ditto
FFIArray!(const(ubyte)) rorm_row_get_data_varbinary(DBRowHandle handle, size_t columnIndex); /// ditto
// todo:
void rorm_row_get_data_date(DBRowHandle handle, size_t columnIndex); /// ditto
void rorm_row_get_data_datetime(DBRowHandle handle, size_t columnIndex); /// ditto
void rorm_row_get_data_timestamp(DBRowHandle handle, size_t columnIndex); /// ditto
void rorm_row_get_data_time(DBRowHandle handle, size_t columnIndex); /// ditto

/// Frees a row handle memory. It may not be read from afterwards anymore.
void rorm_free_row(DBRowHandle handle);

version(none) unittest
{
	import dorm.lib.util;

	DBConnectOptions options = {
		backend: DBBackend.Postgres,
		name: "users".ffi,
		host: "127.0.0.1".ffi
	};
	scope dbHandleAsync = FreeableAsyncResult!DBHandle.make;
	rorm_db_connect(options, dbHandleAsync.callback.expand);
	scope dbHandle = dbHandleAsync.result;
	scope (exit)
		rorm_db_disconnect(dbHandle);

	scope stream = rorm_db_query_stream(dbHandle, "foo".ffi, ["name".ffi].ffi);
	scope (exit)
		rorm_stream_free(stream);

	while (!rorm_stream_empty(stream))
	{
		async_call!rorm_stream_next(stream, (rowResult) {
			writeln("Hello ", rorm_row_get_data_varchar(rowResult.expect, 0));
		}).wait;
	}
}
