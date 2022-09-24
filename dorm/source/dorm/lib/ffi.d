module dorm.lib.ffi;

extern(C):

/**
 * rorm FFI struct definition for arrays and strings (equivalent to D and Rust slices)
 */
struct FFIArray(T)
{
	/**
	 * Pointer to the first item in the slice.
	 */
	T* content;
	/**
	 * The length of the slice. (count of elements)
	 */
	size_t size;

	this(typeof(null))
	{
		this.content = null;
		this.size = 0;
	}

	this(T* content, size_t size)
	{
		this.content = content;
		this.size = size;
	}

	/**
	 * Does a zero-copy conversion of this FFIArray to a D slice. Regular slice
	 * ownership semantics, e.g. variable lifetime, still apply. DIP1000 should
	 * help avoid lifetime issues.
	 */
	inout(T)[] data() inout nothrow pure @nogc return
	{
		return content[0 .. size];
	}
	/// ditto
	alias opSlice = data;

	/**
	 * Zero-copy conversion of a native D slice to an FFIArray. The resulting
	 * FFIArray has the same lifetime as the native D slice, so a stack
	 * allocated slice will also cause the FFIArray to become invalid when
	 * leaving its scope. DIP1000 should help avoid such issues.
	 */
	static FFIArray fromData(return T[] data) nothrow pure @nogc
	{
		return FFIArray(data.ptr, data.length);
	}

	/// ditto
	static FFIArray fromData(size_t n)(return ref T[n] data) nothrow pure @nogc
	{
		return FFIArray(data.ptr, data.length);
	}
}

/// Representation of a string.
alias FFIString = FFIArray!(const(char));

/// helper function to create an FFI slice of a D native array/slice type.
FFIString ffi(string s) { return FFIString.fromData(s); }
/// ditto
FFIArray!T ffi(T)(T[] s) { return FFIArray!T.fromData(s); }
/// ditto
FFIArray!T ffi(T, size_t n)(ref T[n] s) { return FFIArray!T.fromData(s); }

/** 
 * optional value returned by rorm functions.
 */
struct FFIOption(T)
{
	import std.typecons : Nullable;

	/// tagged union type
	enum State
	{
		/// raw_value is not valid (no value inside FFIOption)
		none,
		/// raw_value is the effective value
		some
	}

	/// raw state access
	State state;
	/// raw value access
	T raw_value;

	/// Returns true if the value is set, otherwise false.
	bool opCast(T : bool)() const
	{
		return state != State.none;
	}

	alias asNullable this;
	/// Converts the FFIOption to a std Nullable!T
	Nullable!(T) asNullable() const
	{
		return state == State.none
			? typeof(return).init
			: typeof(return)(raw_value);
	}

	static if (__traits(compiles, { T v = null; }))
	{
		T embedNull() const
		{
			return state == State.none ? T(null) : raw_value;
		}
	}
}

/**
 * Representation of the database backend.
 *
 * This is used to determine the correct driver and the correct dialect to use.
 */
enum DBBackend

{
	/**
	 * This exists to forbid default initializations.
	 *
	 * Using this type will result in an Error with Type.ConfigurationError.
	 */
	Invalid,
	/**
	 * SQLite backend
	 */
	SQLite,
	/**
	 * MySQL / MariaDB backend
	 */
	MySQL,
	/**
	 * Postgres backend
	 */
	Postgres,
}

/**
 * Configuration operation to connect to a database.
 */
struct DBConnectOptions
{
	/// Specifies the driver that will be used.
	DBBackend backend;
	/// Name of the database, in case of `DatabaseBackend.SQLite` name of the file.
	FFIString name;
	/// Host to connect to. Not used in case of `DatabaseBackend.SQLite`.
	FFIString host;
	/// Port to connect to. Not used in case of `DatabaseBackend.SQLite`.
	ushort port;
	/// Username to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	FFIString user;
	/// Password to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	FFIString password;
	/// Minimal connections to initialize upfront. Must not be 0.
	uint minConnections = 1;
	/// Maximum connections that allowed to be created. Must not be 0.
	uint maxConnections = 8;
}

/// Type-safe alias for different handles to void*, to avoid using them in wrong
/// functions accidentally. Try not to use the init value, it's simply a null
/// pointer.
enum DBHandle : void* { init }
/// ditto
enum DBRowHandle : void* { init }
/// ditto
enum DBStreamHandle : void* { init }

/// Represents a (sub-)tree of one or more condition parts.
struct FFICondition
{
	/// tagged union type
	enum Type
	{
		/// A list of [Condition]s, that get expanded to "{} AND {} ..."
		Conjunction,
		/// A list of [Condition]s, that get expanded to "{} OR {} ..."
		Disjunction,
		/// Representation of a unary condition.
		UnaryCondition,
		/// Representation of a binary condition.
		BinaryCondition,
		/// Representation of a ternary condition.
		TernaryCondition,
		/// Representation of a value.
		Value
	}
	/// ditto
	Type type;

	union
	{
		/// Correpsonding value for Type.Conjunction
		FFIArray!FFICondition conjunction;
		/// Correpsonding value for Type.Disjunction
		FFIArray!FFICondition disjunction;
		/// Correpsonding value for Type.UnaryCondition
		FFIUnaryCondition unaryCondition;
		/// Correpsonding value for Type.BinaryCondition
		FFIBinaryCondition binaryCondition;
		/// Correpsonding value for Type.TernaryCondition
		FFITernaryCondition ternaryCondition;
		/// Correpsonding value for Type.Value
		ConditionValue value;
	}
}

/// This condition subtype represents all available unary conditions.
/// (operations with a single operand)
struct FFIUnaryCondition
{
	/// tagged union type
	enum Type
	{
		/// Representation of "{} IS NULL" in SQL
		IsNull,
		/// Representation of "{} IS NOT NULL" in SQL
		IsNotNull,
		/// Representation of "EXISTS {}" in SQL
		Exists,
		/// Representation of "NOT EXISTS {}" in SQL
		NotExists,
		/// Representation of "NOT {}" in SQL
		Not
	}
	/// ditto
	Type type;

	/// The operand for any unary condition on which to operate using the type.
	FFICondition* condition;
}

/// This condition subtype represents all available binary conditions.
/// (operations with two operands)
struct FFIBinaryCondition
{
	/// tagged union type
	enum Type
	{
		/// Representation of "{} = {}" in SQL
		Equals,
		/// Representation of "{} <> {}" in SQL
		NotEquals,
		/// Representation of "{} > {}" in SQL
		Greater,
		/// Representation of "{} >= {}" in SQL
		GreaterOrEquals,
		/// Representation of "{} < {}" in SQL
		Less,
		/// Representation of "{} <= {}" in SQL
		LessOrEquals,
		/// Representation of "{} LIKE {}" in SQL
		Like,
		/// Representation of "{} NOT LIKE {}" in SQL
		NotLike,
		/// Representation of "{} REGEXP {}" in SQL
		Regexp,
		/// Representation of "{} NOT REGEXP {}" in SQL
		NotRegexp,
		/// Representation of "{} IN {}" in SQL
		In,
		/// Representation of "{} NOT IN {}" in SQL
		NotIn
	}
	/// ditto
	Type type;

	/// The left-hand-side operand on which to operate based on the type.
	FFICondition* lhs;
	/// The right-hand-side operand on which to operate based on the type.
	FFICondition* rhs;
}

/// This condition subtype represents all available ternary conditions.
/// (operations with three operands)
struct FFITernaryCondition
{
	/// tagged union type
	enum Type
	{
		/// Representation of "{} BETWEEN {} AND {}" in SQL
		Between,
		/// Representation of "{} NOT BETWEEN {} AND {}" in SQL
		NotBetween
	}
	/// ditto
	Type type;

	/// The first operand on which to operate based on the type.
	FFICondition* first;
	/// The second operand on which to operate based on the type.
	FFICondition* second;
	/// The third operand on which to operate based on the type.
	FFICondition* third;
}

/// Represents a leaf node in a condition tree, effectively inserting a static
/// value like a string, identifier or number.
struct ConditionValue
{
	/// tagged union type
	enum Type
	{
		/// This represents `NULL` in SQL.
		Null,
		/// Representation of an identifier, e.g. a column name.
		/// The value will not be escaped, so do not pass unchecked data to it.
		Identifier,
		/// The value represents a string, being escaped (e.g. quoted)
		String,
		/// The value represents a 64 bit signed integer
		I64,
		/// The value represents a 32 bit signed integer
		I32,
		/// The value represents a 16 bit signed integer
		I16,
		/// The value represents a boolean value (true or false)
		Bool,
		/// The value represents a 64 bit floating point value
		F64,
		/// The value represents a 32 bit floating point value
		F32,
		/// Binary representation
		Binary,
	}
	/// ditto
	Type type;

	union
	{
		/// Corresponds to Type.Identifier
		FFIString identifier;
		/// Corresponds to Type.String
		FFIString string;
		/// Corresponds to Type.I64
		long i64;
		/// Corresponds to Type.I32
		int i32;
		/// Corresponds to Type.I16
		short i16;
		/// Corresponds to Type.Bool
		bool boolean;
		/// Corresponds to Type.F64
		double f64;
		/// Corresponds to Type.F32
		float f32;
		/// Corresponds to Type.Binary
		FFIArray!ubyte binary;
	}
}

/**
 * Error struct passed into rorm callbacks. Generally this may not escape the
 * callback, so it must always be used with scope, unless otherwise documented.
 *
 * Usually it should not be neccessary to use this directly from user code.
 */
struct RormError
{
	/**
	 * Representation of all error codes.
	 */
	enum Tag
	{
		/**
		 * Everything's fine, nothing to worry about. Other result data passed
		 * into callbacks, such as returned handles, may only be used and freed
		 * if there is no error.
		 */
		NoError,
		/**
		 * Runtime was destroyed or never created and can therefore not be
		 * accessed.
		 */
		MissingRuntimeError,
		/**
		 * An error occurred accessing the runtime.
		 */
		RuntimeError,
		/**
		 * String with invalid UTF8 content was passed into the function.
		 */
		InvalidStringError,
		/**
		 * Configuration error
		 */
		ConfigurationError,
		/**
		 * Database error
		 */
		DatabaseError,
		/**
		 * There are no rows left in the stream
		 */
		NoRowsLeftInStream,
		/**
		 * Column could not be converted in the given type
		 */
		ColumnDecodeError,
		/**
		 * Column was not found in row
		 */
		ColumnNotFoundError,
		/**
		 * The index in the row was out of bounds
		 */
		ColumnIndexOutOfBoundsError,
	}
	/// ditto
	Tag tag;

	union {
		/// Error message for tag == Tag.RuntimeError
		FFIString runtime_error;
		/// Error message for tag == Tag.ConfigurationError
		FFIString configuration_error;
		/// Error message for tag == Tag.DatabaseError
		FFIString database_error;
	}

	/**
	 * Returns false only when there is no error, otherwise true.
	 *
	 * Examples:
	 * ---
	 * void myCallback(Handle data, Error error) {
	 *     if (error) throw error.makeException;
	 *     // only start using `data` from this point on
	 * }
	 * ---
	 */
	bool opCast() const nothrow @nogc @safe
	{
		return tag != Tag.NoError;
	}

	/// Makes a human readable exception that can be thrown or returns `null` if
	/// there is no error.
	Exception makeException() const nothrow @safe
	{
		import std.conv : text;
		import std.utf : UTFException;

		final switch (tag)
		{
			case Tag.NoError: return null;
			case Tag.MissingRuntimeError:
				return new Exception(
					"Runtime has not been created or has been destroyed, use `mixin SetupDormRuntime;` in your application code");
			case Tag.RuntimeError:
				return new Exception(
					text("A runtime error has occurred: ", (() @trusted => this.runtime_error.data)()));
			case Tag.InvalidStringError:
				return new UTFException(
					"an invalid string has been passed into a dorm function, perhaps corrupted memory? (submit a bug in this case)");
			case Tag.ConfigurationError:
				return new Exception(
					text("passed invalid configuration: ", (() @trusted => this.configuration_error.data)()));
			case Tag.DatabaseError:
				return new Exception(
					text("database error: ", (() @trusted => this.database_error.data)()));
			case Tag.NoRowsLeftInStream:
				return new Exception("There are no rows left in the stream");
			case Tag.ColumnDecodeError:
				return new Exception("Column could not be converted in the given type");
			case Tag.ColumnNotFoundError:
				return new Exception("Column was not found in row");
			case Tag.ColumnIndexOutOfBoundsError:
				return new Exception("The index in the row was out of bounds");
		}
	}
}

/**
 * This function is used to initialize and start the async runtime.
 *
 * It is needed as rorm is completely written asynchronously.
 *
 * **Important**: Do not forget to stop the runtime using $(LREF rorm_runtime_shutdown)!
 *
 * For user code, use `mixin SetupDormRuntime;` instead.
 *
 * This function is called completely synchronously.
 */
void rorm_runtime_start(RuntimeStartCallback callback, void* context);
/// ditto
alias RuntimeStartCallback = extern(C) void function(void* context, scope RormError);

/**
 * Shutdown the runtime.
 *
 * Returns:
 * - If no runtime is currently existing, a `Error.Type.MissingRuntimeError` will be returned.
 * - If the runtime could not be locked, a `Error.Type.RuntimeError` containing further information will be returned.
 *
 * Params:
 *     timeoutMsecs = Specify the amount of time to wait in milliseconds.
 *     callback = Callback to call when finished, only passing in error information.
 *     context = context pointer to pass through as-is into the callback.
 *
 * This function is called completely synchronously.
 */
void rorm_runtime_shutdown(ulong timeoutMsecs, RuntimeShutdownCallback callback, void* context);
/// ditto
alias RuntimeShutdownCallback = extern(C) void function(void* context, scope RormError);

/**
 * Connect to the database using the provided $(LREF DBConnectOptions).
 *
 * To free the handle, use [rorm_db_free].
 *
 * Params:
 *     options = connection and behavior options for the DB connection handle
 *     callback = Callback that's called when the connection is established
 *         either successfully or unsuccessfully. The callback parameters are:
 *       data = the context pointer as passed into the function call by the user.
 *       handle = if error is NoError, a valid handle to a DB connection,
 *         otherwise not a valid handle and should not be freed.
 *       error = if not successful, error information what exactly happened.
 *     context = context pointer to pass through as-is into the callback.
 *
 * This function is running in an asynchronous context.
 */
void rorm_db_connect(DBConnectOptions options, DBConnectCallback callback, void* context);
/// ditto
alias DBConnectCallback = extern(C) void function(void* context, DBHandle handle, scope RormError error) nothrow;

/**
 * Closes all of the database connections and frees the handle. No database
 * operations may be pending when calling this!
 *
 * Takes the pointer to the database instance. Must not be called with an
 * invalid handle. (when the `error` field is set to anything other than NoError)
 *
 * **Important**: Do not call this function more than once!
 *
 * This function is called completely synchronously.
 */
void rorm_db_free(DBHandle handle);

/**
 * This function queries the database given the provided parameters.
 *
 * Returns: a pointer to the created stream.
 *
 * Parameters:
 *     box = Reference to the Database, provided by $(LREF rorm_db_connect).
 *     model = Name of the table to query.
 *     columns = Array of columns to retrieve from the database.
 *     condition = Pointer to a $(LREF FFICondition).
 *     callback = callback function. Takes the `context`, a stream handle and an
 *         error that must be checked first.
 *     context = context pointer to pass through as-is into the callback.
 *
 * This function is called completely synchronously.
 */
DBStreamHandle rorm_db_query_stream(DBHandle handle,
	FFIString model,
	FFIArray!FFIString columns,
	const(FFICondition)* conditionTree,
	DbQueryStreamCallback callback,
	void* context);
/// ditto
alias DbQueryStreamCallback = extern(C) void function(void* context, DBStreamHandle stream, scope RormError);

/**
 * Frees the stream given as parameter. The stream must be in a freeable state.
 *
 * **Important**: Do not call this function more than once or with an invalid
 * stream!
 *
 * This function is called completely synchronously.
 */
void rorm_stream_free(DBStreamHandle handle);

/**
 * Fetches the next row from the stream. Must not be called in parallel on the
 * same stream. Returns Error.NoRowsLeftInStream when the stream is finished.
 *
 * Params:
 *     stream = handle of a queried stream. (must have ownership)
 *     callback = called when a row is fetched, contains either an error that
 *         can be accessed within the callback or a row handle that can be moved
 *         out the callback, but must be freed with $(LREF rorm_row_free).
 *     context = context pointer to pass through as-is into the callback.
 *
 * This function is running in an asynchronous context.
 */
void rorm_stream_get_row(DBStreamHandle stream, scope DbStreamGetRowCallback callback, void* context);
/// ditto
alias DbStreamGetRowCallback = extern(C) void function(void* context, DBRowHandle row, scope RormError) nothrow;

/**
 * Frees the row handle given as parameter.
 *
 * **Important**: Do not call this function more than once or with an invalid
 * row handle!
 *
 * This function is called completely synchronously.
 */
void rorm_row_free(DBRowHandle row);

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
short rorm_row_get_i16(DBRowHandle handle, FFIString column, ref RormError ref_error);
int rorm_row_get_i32(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
long rorm_row_get_i64(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
float rorm_row_get_f32(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
double rorm_row_get_f64(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
bool rorm_row_get_bool(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIString rorm_row_get_str(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIArray!(const ubyte) rorm_row_get_binary(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!short rorm_row_get_null_i16(DBRowHandle handle, FFIString column, ref RormError ref_error);
FFIOption!int rorm_row_get_null_i32(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!long rorm_row_get_null_i64(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!float rorm_row_get_null_f32(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!double rorm_row_get_null_f64(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!bool rorm_row_get_null_bool(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!FFIString rorm_row_get_null_str(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
FFIOption!(FFIArray!(const ubyte)) rorm_row_get_null_binary(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto

version (none)
{
	// todo:
	FFIArray!(const(ubyte)) rorm_row_get_varbinary(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
	void rorm_row_get_date(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
	void rorm_row_get_datetime(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
	void rorm_row_get_timestamp(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
	void rorm_row_get_time(DBRowHandle handle, FFIString column, ref RormError ref_error); /// ditto
}


unittest
{
	import dorm.lib.util;

	sync_call!rorm_runtime_start();
	scope (exit)
		sync_call!rorm_runtime_shutdown(1000);

	DBConnectOptions options = {
		backend: DBBackend.SQLite,
		name: "test_read.sqlite3".ffi,
	};
	scope dbHandleAsync = FreeableAsyncResult!DBHandle.make;
	rorm_db_connect(options, dbHandleAsync.callback.expand);
	scope dbHandle = dbHandleAsync.result;
	scope (exit)
		rorm_db_free(dbHandle);

	scope stream = sync_call!rorm_db_query_stream(dbHandle, "foo".ffi, ["name".ffi, "notes".ffi].ffi, null);
	scope (exit)
		rorm_stream_free(stream);

	import std.stdio;
	writeln("| Name \t| Notes \t|");

	Exception e;
	while (true)
	{
		scope rowHandleAsync = RowHandleState(FreeableAsyncResult!DBRowHandle.make);
		rorm_stream_get_row(stream, &rowCallback, cast(void*)&rowHandleAsync);
		scope rowHandle = rowHandleAsync.result;
		if (rowHandleAsync.done)
			break;
		scope (exit)
			rorm_row_free(rowHandle);

		RormError rowError;
		auto name = rorm_row_get_str(rowHandle, "name".ffi, rowError);
		auto notes = rorm_row_get_null_str(rowHandle, "notes".ffi, rowError).embedNull;
		if (rowError)
		{
			e = rowError.makeException;
			break;
		}
		writefln("| %s\t| %s\t|", name[], notes[]);
	}

	// while (!rorm_stream_empty(stream))
	// {
	// 	async_call!rorm_stream_next(stream, (rowResult) {
	// 		writeln("Hello ", rorm_row_get_data_varchar(rowResult.expect, FFIString)column, ref RormError ref_error
	// 	}).wait;
	// }
}

version (unittest)
{
	import dorm.lib.util;

	private struct RowHandleState
	{
		FreeableAsyncResult!DBRowHandle impl;
		alias impl this;
		bool done;
	}

	extern(C) private static void rowCallback(void* data, DBRowHandle result, scope RormError error) nothrow
	{
		auto res = cast(RowHandleState*)data;
		if (error.tag == RormError.Tag.NoRowsLeftInStream)
			res.done = true;
		else if (error)
			res.error = error.makeException;
		else
			res.raw_result = result;
		res.event.set();
	}
}
