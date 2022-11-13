module dorm.api.db;

import dorm.declarative;
import dorm.declarative.conversion;
import dorm.lib.util;
import dorm.types;
import ffi = dorm.lib.ffi;

import std.algorithm : move;
import std.conv : text, to;
import std.datetime : Clock, Date, DateTime, DateTimeException, SysTime, TimeOfDay, UTC;
import std.meta;
import std.range.primitives;
import std.traits;

import core.attribute;
import core.time;

public import dorm.types : DormPatch;
public import dorm.lib.ffi : DBBackend;

public import dorm.api.condition;

static if (!is(typeof(mustuse)))
	enum mustuse;

/**
 * Configuration operation to connect to a database.
 */
struct DBConnectOptions
{
@safe:
	/// Specifies the driver that will be used.
	DBBackend backend;
	/// Name of the database, in case of `DatabaseBackend.SQLite` name of the file.
	string name;
	/// Host to connect to. Not used in case of `DatabaseBackend.SQLite`.
	string host;
	/// Port to connect to. Not used in case of `DatabaseBackend.SQLite`.
	ushort port;
	/// Username to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	string user;
	/// Password to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	string password;
	/// Minimal connections to initialize upfront. Must not be 0.
	uint minConnections = ffi.DBConnectOptions.init.minConnections;
	/// Maximum connections that allowed to be created. Must not be 0.
	uint maxConnections = ffi.DBConnectOptions.init.maxConnections;
}

/**
 * High-level wrapper around a database. Through the driver implementation layer
 * this handles connection pooling and distributes work across a thread pool
 * automatically.
 *
 * Use the (UFCS) methods
 *
 * - $(LREF select)
 * - $(LREF insert)
 *
 * to access the database.
 *
 * This struct cannot be copied, to pass it around, use `ref`. Once the struct
 * goes out of scope or gets unset, the connection to the database will be freed.
 */
struct DormDB
{
@safe:
	private ffi.DBHandle handle;

	@disable this();

	/**
	 * Performs a Database connection (possibly in another thread) and returns
	 * the constructed DormDB handle once connected.
	 */
	this(DBConnectOptions options) @trusted
	{
		// TODO: think of how to make async waiting configurable, right now the thread is just blocked
		auto ffiOptions = options.ffiInto!(ffi.DBConnectOptions);

		scope dbHandleAsync = FreeableAsyncResult!(ffi.DBHandle).make;
		ffi.rorm_db_connect(ffiOptions, dbHandleAsync.callback.expand);
		handle = dbHandleAsync.result;
	}

	~this() @trusted
	{
		if (handle)
		{
			ffi.rorm_db_free(handle);
			handle = null;
		}
	}

	@disable this(this);

	DormTransaction startTransaction() return
	{
		ffi.DBTransactionHandle handle;
		(() @trusted {
			auto ctx = FreeableAsyncResult!(ffi.DBTransactionHandle).make;
			ffi.rorm_db_start_transaction(this.handle, ctx.callback.expand);
			handle = ctx.result();
		})();
		return DormTransaction(&this, handle);
	}

	void insert(T)(T value)
	if (!is(T == U[], U))
	{
		return (() @trusted => insertImpl!true(handle, (&value)[0 .. 1], null))();
	}

	void insert(T)(scope T[] value)
	{
		return insertImpl!false(handle, value, null);
	}

	/**
	 * This function executes a raw SQL statement.
	 *
	 * Iterate over the result using `foreach`.
	 *
	 * Statements are executed as prepared statements, if possible.
	 *
	 * To define placeholders, use `?` in SQLite and MySQL and $1, $n in Postgres.
	 * The corresponding parameters are bound in order to the query.
	 *
	 * The number of placeholder must match with the number of provided bind
	 * parameters.
	 *
	 * To include the statement in a transaction specify `transaction` as a valid
	 * Transaction. As the Transaction needs to be mutable, it is important to not
	 * use the Transaction anywhere else until the callback is finished.
	 *
	 * Params:
	 *     queryString = SQL statement to execute.
	 *     bindParams = Parameters to fill into placeholders of `queryString`.
	 */
	RawSQLIterator rawSQL(scope return const(char)[] queryString, scope return ffi.FFIValue[] bindParams = null) return pure
	{
		return RawSQLIterator(&this, null, queryString, bindParams);
	}
}

///
@mustuse struct RawSQLIterator
{
	private DormDB* db;
	private ffi.DBTransactionHandle tx;
	private const(char)[] queryString;
	private ffi.FFIValue[] bindParams;
	private size_t rowCountImpl = -1;

	/// Returns the number of rows, only valid inside the foreach.
	size_t rowCount()
	{
		assert(rowCountImpl != -1, "Calling rowCount is only valid inside the foreach / opApply");
		return rowCountImpl;
	}

	/// Starts a new query and iterates all the results on each foreach call.
	int opApply(scope int delegate(scope RawRow row) dg) @trusted
	{
		scope (exit)
			rowCountImpl = -1;
		assert(rowCountImpl == -1, "Don't iterate over the same RawSQLIterator on multiple threads!");

		int result = 0;
		auto ctx = FreeableAsyncResult!(void delegate(scope ffi.FFIArray!(ffi.DBRowHandle))).make;
		ctx.forward_callback = (scope rows) {
			rowCountImpl = rows.size;
			foreach (row; rows[])
			{
				result = dg(RawRow(row));
				if (result)
					break;
			}
		};
		ffi.rorm_db_raw_sql(db.handle,
			tx,
			ffi.ffi(queryString),
			ffi.ffi(bindParams),
			ctx.callback.expand);
		ctx.result();
		return result;
	}

	/// Runs the raw SQL query, discarding results (throwing on error)
	void exec()
	{
		assert(rowCountImpl == -1, "Don't iterate over the same RawSQLIterator on multiple threads!");

		auto ctx = FreeableAsyncResult!(void delegate(scope ffi.FFIArray!(ffi.DBRowHandle))).make;
		ctx.forward_callback = (scope rows) {};
		ffi.rorm_db_raw_sql(db.handle,
			tx,
			ffi.ffi(queryString),
			ffi.ffi(bindParams),
			ctx.callback.expand);
		ctx.result();
	}
}

/// Allows column access on a raw DB row as returned by `db.rawSQL`.
struct RawRow
{
	private ffi.DBRowHandle row;

	@disable this(this);

	private static template ffiConvPrimitive(T)
	{
		static if (is(T == short))
			alias ffiConvPrimitive = ffi.rorm_row_get_i16;
		else static if (is(T == int))
			alias ffiConvPrimitive = ffi.rorm_row_get_i32;
		else static if (is(T == long))
			alias ffiConvPrimitive = ffi.rorm_row_get_i64;
		else static if (is(T == float))
			alias ffiConvPrimitive = ffi.rorm_row_get_f32;
		else static if (is(T == double))
			alias ffiConvPrimitive = ffi.rorm_row_get_f64;
		else static if (is(T == bool))
			alias ffiConvPrimitive = ffi.rorm_row_get_bool;
		else
			static assert(false, "Unsupported column type: " ~ T.stringof);
	}

	/// Gets the value of the column at the given column name assuming it is of
	/// the given type. If the value is not of the given type, an exception will
	/// be thrown.
	///
	/// Supported types:
	/// - any string type (auto-converted from strings / varchar)
	/// - `ubyte[]` for binary data
	/// - `short`, `int`, `long`, `float`, `double`, `bool`
	///
	/// For nullable values, use $(LREF opt) instead.
	T get(T)(scope const(char)[] column)
	{
		auto ffiColumn = ffi.ffi(column);
		ffi.RormError error;
		T result;

		static if (isSomeString!T)
		{
			auto slice = ffi.rorm_row_get_str(row, ffiColumn, error);
			if (!error)
			{
				static if (is(T : char[]))
					result = cast(T)slice[].dup;
				else
					result = slice[].to!T;
			}
		}
		else static if (is(T : ubyte[]))
		{
			auto slice = ffi.rorm_row_get_binary(row, ffiColumn, error);
			if (!error)
				result = cast(T)slice[].dup;
		}
		else
		{
			alias fn = ffiConvPrimitive!T;
			result = fn(row, ffiColumn, error);
		}

		if (error)
			throw error.makeException(" (in column '" ~ column.idup ~ "')");
		return result;
	}

	private static template ffiConvOptionalPrimitive(T)
	{
		static if (is(T == short))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_i16;
		else static if (is(T == int))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_i32;
		else static if (is(T == long))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_i64;
		else static if (is(T == float))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_f32;
		else static if (is(T == double))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_f64;
		else static if (is(T == bool))
			alias ffiConvOptionalPrimitive = ffi.rorm_row_get_null_bool;
		else
			static assert(false, "Unsupported column type: " ~ T.stringof);
	}

	/// Same as get, wraps primitives inside Nullable!T. Strings and ubyte[]
	/// binary arrays will return `null` (checkable with `is null`), but
	/// otherwise simply be embedded.
	auto opt(T)(scope const(char)[] column)
	{
		auto ffiColumn = ffi.ffi(column);
		ffi.RormError error;

		static if (isSomeString!T)
		{
			auto slice = ffi.rorm_row_get_null_str(row, ffiColumn, error);
			if (!error)
			{
				if (slice.isNull)
					return null;
				static if (is(T : char[]))
					return cast(T)slice.raw_value[].dup;
				else
					return slice.raw_value[].to!T;
			}
			else
				throw error.makeException(" (in column '" ~ column.idup ~ "')");
		}
		else static if (is(T : ubyte[]))
		{
			auto slice = ffi.rorm_row_get_null_binary(row, ffiColumn, error);
			if (slice.isNull)
				return null;
			if (!error)
				return cast(T)slice.raw_value[].dup;
			else
				throw error.makeException(" (in column '" ~ column.idup ~ "')");
		}
		else
		{
			Nullable!T result;
			alias fn = ffiConvOptionalPrimitive!T;
			auto opt = fn(row, ffiColumn, error);
			if (error)
				throw error.makeException(" (in column '" ~ column.idup ~ "')");
			if (!opt.isNull)
				result = opt.raw_value;
			return result;
		}
	}
}

/**
 * Wrapper around a Database transaction. Most methods that can be used on a
 * DormDB can also be used on a transaction.
 *
 * Performs a rollback when going out of scope and wasn't committed or rolled
 * back explicitly.
 */
struct DormTransaction
{
@safe:
	private DormDB* db;
	private ffi.DBTransactionHandle txHandle;

	@disable this(this);

	~this()
	{
		if (txHandle)
		{
			rollback();
		}
	}

	/// Commits this transaction, so the changes are recorded to the current
	/// database state.
	void commit()
	{
		scope (exit) txHandle = null;
		(() @trusted {
			auto ctx = FreeableAsyncResult!void.make;
			ffi.rorm_transaction_commit(txHandle, ctx.callback.expand);
			ctx.result();
		})();
	}

	/// Rolls back this transaction, so the DB changes are reverted to before
	/// the transaction was started.
	void rollback()
	{
		scope (exit) txHandle = null;
		(() @trusted {
			auto ctx = FreeableAsyncResult!void.make;
			ffi.rorm_transaction_rollback(txHandle, ctx.callback.expand);
			ctx.result();
		})();
	}

	/// Transacted variant of $(LREF DormDB.insert).
	void insert(T)(T value)
	{
		return (() @trusted => insertImpl!true(db.handle, (&value)[0 .. 1], txHandle))();
	}

	void insert(T)(scope T[] value)
	{
		return insertImpl!false(db.handle, value, txHandle);
	}
}

private string makePatchAccessPrefix(Patch, DB)()
{
	string ret;
	static if (!is(Patch == DB)
		&& is(__traits(parent, Patch) == DB))
	{
		static foreach (i, field; DB.tupleof)
		{
			static if (is(typeof(field) == Patch))
			{
				static foreach_reverse (j, field; DB.tupleof)
					static if (is(typeof(field) == Patch))
						static assert(i == j, "Multiple implicit "
							~ Patch.stringof ~ " patches on the same "
							~ DB.stringof ~ " Model class!");

				ret = DB.tupleof[i].stringof ~ ".";
			}
		}
	}
	return ret;
}

private void insertImpl(bool single, T)(scope ffi.DBHandle handle, scope T[] value, ffi.DBTransactionHandle transaction) @safe
{
	import core.lifetime;
	alias DB = DBType!T;

	enum patchAccessPrefix = makePatchAccessPrefix!(T, DB);

	static stripPrefix(string s)
	{
		return patchAccessPrefix.length && s.length > patchAccessPrefix.length
			&& s[0 .. patchAccessPrefix.length] == patchAccessPrefix
			? s[patchAccessPrefix.length .. $] : s;
	}

	enum NumColumns = {
		int used;
		static foreach (field; DormFields!DB)
			static if (is(typeof(mixin("value[0]." ~ stripPrefix(field.sourceColumn)))) || field.hasConstructValue)
				used++;
		return used;
	}();

	ffi.FFIString[NumColumns] columns;
	static if (single)
	{
		ffi.FFIValue[NumColumns][1] values;
	}
	else
	{
		ffi.FFIValue[NumColumns][] values;
		values.length = value.length;

		if (!values.length)
			return;
	}

	int used;

	static if (!is(T == DB))
	{
		auto validatorObject = new DB();
		static if (!single)
		{
			DB validatorCopy;
			if (values.length > 1)
				(() @trusted => copyEmplace(validatorObject, validatorCopy))();
		}
	}

	static foreach (field; DormFields!DB)
	{{
		static if (is(typeof(mixin("value[0]." ~ stripPrefix(field.sourceColumn)))))
		{
			columns[used] = ffi.ffi(field.columnName);
			foreach (i; 0 .. values.length)
				values[i][used] = conditionValue!field(mixin("value[i]." ~ stripPrefix(field.sourceColumn)));
			used++;
		}
		else static if (field.hasConstructValue)
		{
			// filled in by constructor
			columns[used] = ffi.ffi(field.columnName);
			foreach (i; 0 .. values.length)
			{
				static if (is(T == DB))
					values[i][used] = conditionValue!field(mixin("value[i]." ~ field.sourceColumn));
				else
					values[i][used] = conditionValue!field(mixin("validatorObject." ~ stripPrefix(field.sourceColumn)));
			}
			used++;
		}
		else static if (field.hasGeneratedDefaultValue)
		{
			// OK
		}
		else static if (!is(T == DB))
			static assert(false, "Trying to insert a patch " ~ T.stringof
				~ " into " ~ DB.stringof ~ ", but it is missing the required field "
				~ stripPrefix(field.sourceReferenceName) ~ "! "
				~ "Fields with auto-generated values may be omitted in patch types. "
				~ ModelFormat.Field.humanReadableGeneratedDefaultValueTypes);
		else
			static assert(false, "wat? (defined DormField not found inside the Model class that defined it)");
	}}

	assert(used == NumColumns);

	static if (is(T == DB))
	{
		foreach (i; 0 .. values.length)
		{
			auto brokenFields = value[i].runValidators();

			string error;
			foreach (field; brokenFields)
			{
				static if (single)
					error ~= "Field `" ~ field.sourceColumn ~ "` defined in "
						~ field.definedAt.toString ~ " failed user validation.";
				else
					error ~= "row[" ~ i.to!string
						~ "] field `" ~ field.sourceColumn ~ "` defined in "
						~ field.definedAt.toString ~ " failed user validation.";
			}
			if (error.length)
				throw new Exception(error);
		}
	}
	else
	{
		foreach (i; 0 .. values.length)
		{
			static if (!single)
				if (i != 0)
					(() @trusted => copyEmplace(validatorCopy, validatorObject))();

			validatorObject.applyPatch(value[i]);
			auto brokenFields = validatorObject.runValidators();

			string error;
			foreach (field; brokenFields)
			{
				switch (field.columnName)
				{
					static foreach (sourceField; DormFields!DB)
					{
						static if (is(typeof(mixin("value[i]." ~ stripPrefix(sourceField.sourceColumn)))))
						{
							case sourceField.columnName:
						}
					}
					static if (single)
						error ~= "Field `" ~ field.sourceColumn ~ "` defined in "
							~ field.definedAt.toString ~ " failed user validation.";
					else
						error ~= "row[" ~ i.to!string
							~ "] field `" ~ field.sourceColumn ~ "` defined in "
							~ field.definedAt.toString ~ " failed user validation.";
					break;
				default:
					break;
				}
			}

			if (error.length)
				throw new Exception(error);
		}
	}


	(() @trusted {
		auto ctx = FreeableAsyncResult!void.make;
		static if (single)
		{
			ffi.rorm_db_insert(handle,
				transaction,
				ffi.ffi(DormLayout!DB.tableName),
				ffi.ffi(columns),
				ffi.ffi(values[0]), ctx.callback.expand);
		}
		else
		{
			auto rows = new ffi.FFIArray!(ffi.FFIValue)[values.length];
			foreach (i; 0 .. values.length)
				rows[i] = ffi.ffi(values[i]);

			ffi.rorm_db_insert_bulk(handle,
				transaction,
				ffi.ffi(DormLayout!DB.tableName),
				ffi.ffi(columns),
				ffi.ffi(rows), ctx.callback.expand);
		}
		ctx.result();
	})();
}

// defined this as global so that we can pass `Foo.fieldName` as alias argument,
// to have it be selected.
static SelectOperation!(DBType!(Selection), SelectType!(Selection)) select(Selection...)(return ref const DormDB db) @trusted
{
	return typeof(return)(&db, null);
}

static SelectOperation!(DBType!(Selection), SelectType!(Selection)) select(Selection...)(return ref const DormTransaction tx) @trusted
{
	return typeof(return)(tx.db, tx.txHandle);
}

private struct ConditionBuilderData
{
	@disable this(this);

	ffi.FFICondition[64] conditionStack;
	size_t conditionStackIndex;
	JoinInformation joinInformation;
}

struct ConditionBuilder(T)
{
	private ConditionBuilderData* builderData;

	static foreach (i, member; T.tupleof)
	{
		static if (hasDormField!(T, T.tupleof[i].stringof))
		{
			static if (DormField!(T, T.tupleof[i].stringof).isForeignKey)
			{
				mixin("ForeignModelConditionBuilderField!(typeof(T.tupleof[i]), DormField!(T, T.tupleof[i].stringof)) ",
					T.tupleof[i].stringof,
					"() return { return typeof(return)(builderData, `",
					DormField!(T, T.tupleof[i].stringof).columnName,
					"`); }");
			}
			else
			{
				mixin("ConditionBuilderField!(typeof(T.tupleof[i]), DormField!(T, T.tupleof[i].stringof)) ",
					T.tupleof[i].stringof,
					" = ConditionBuilderField!(typeof(T.tupleof[i]), DormField!(T, T.tupleof[i].stringof))(`",
					DormField!(T, T.tupleof[i].stringof).columnName,
					"`);");
			}
		}
	}

	static if (__traits(allMembers, NotConditionBuilder!T).length > 1)
		NotConditionBuilder!T not;
	else
		void not()() { static assert(false, "Model " ~ T.stringof
			~ " has no fields that can be used with .not"); }

	auto opDispatch(string member)()
	{
		import std.string : join;

		pragma(msg, supplErrorPrefix ~ "cannot access condition field '" ~ member ~ "'. Available members are: "
			~ [__traits(allMembers, ConditionBuilder)][0 .. ConditionBuilder.tupleof.length].join(", "));
		static assert(false);
	}
}

private enum supplErrorPrefix = "           \x1B[1;31mDORM Error:\x1B[m ";

struct NotConditionBuilder(T)
{
	static foreach (i, member; T.tupleof)
	{
		static if (is(typeof(T.tupleof[i]) : bool))
		{
			mixin("Condition ",
				T.tupleof[i].stringof,
				"() { return Condition(UnaryCondition(UnaryConditionType.Not,
					makeConditionIdentifier(`",
				DormField!(T, T.tupleof[i].stringof).columnName,
				"`))); }");
		}
	}

	auto opDispatch(string member)()
	{
		import std.string : join;

		pragma(msg, supplErrorPrefix ~ "cannot access negated condition field '" ~ member ~ "'. Available members are: "
			~ [__traits(allMembers, NotConditionBuilder)][0 .. $ - 1].join(", "));
		static assert(false);
	}
}

private Condition* makeConditionIdentifier(T)(T value) @safe
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionIdentifier(value));
}

private Condition* makeConditionConstant(ModelFormat.Field fieldInfo, T)(T value) @safe
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionValue!fieldInfo(value));
}

struct ForeignModelConditionBuilderField(ModelRef, ModelFormat.Field field)
{
	alias RefDB = ModelRef.TModel;

	private ConditionBuilderData* data;
	private string columnName;

	this(ConditionBuilderData* data, string columnName) @safe
	{
		this.data = data;
		this.columnName = columnName;
	}

	string ensureJoined() @trusted
	{
		auto ji = &data.joinInformation;
		string fkName = field.columnName;
		auto exist = fkName in ji.joinedTables;
		if (exist)
		{
			return *exist;
		}
		else
		{
			string placeholder = "_" ~ ji.joinedTables.length.to!string;
			ji.joinedTables[fkName] = placeholder;
			ffi.FFICondition condition;
			condition.type = ffi.FFICondition.Type.BinaryCondition;
			condition.binaryCondition.type = ffi.FFIBinaryCondition.Type.Equals;
			auto sides = data.conditionStack[data.conditionStackIndex ..
				data.conditionStackIndex += 2];
			assert(sides.length == 2);
			sides[0].type = ffi.FFICondition.Type.Value;
			sides[0].value = conditionIdentifier(placeholder ~ "." ~ ModelRef.primaryKeyField.columnName);
			sides[1].type = ffi.FFICondition.Type.Value;
			sides[1].value = conditionIdentifier(field.columnName);
			condition.binaryCondition.lhs = &sides[0];
			condition.binaryCondition.rhs = &sides[1];

			ji.joins ~= ffi.FFIJoin(
				ffi.FFIJoinType.join,
				ffi.ffi(DormLayout!RefDB.tableName),
				ffi.ffi(placeholder),
				ffi.FFIOption!(ffi.FFICondition)(condition)
			);
			return placeholder;
		}
	}

	static foreach (i, member; RefDB.tupleof)
	{
		static if (__traits(isSame, ModelRef.primaryKeyAlias, member))
		{
			mixin("ConditionBuilderField!(ModelRef.PrimaryKeyType, field) ",
				RefDB.tupleof[i].stringof,
				" = ConditionBuilderField!(ModelRef.PrimaryKeyType, field)(`",
				field.columnName,
				"`);");
		}
		else static if (hasDormField!(RefDB, RefDB.tupleof[i].stringof))
		{
			mixin("ConditionBuilderField!(typeof(RefDB.tupleof[i]), DormField!(RefDB, RefDB.tupleof[i].stringof)) ",
				RefDB.tupleof[i].stringof,
				"() @safe return { string placeholder = ensureJoined(); return typeof(return)(placeholder ~ `.",
				DormField!(RefDB, RefDB.tupleof[i].stringof).columnName,
				"`); }");
		}
	}
}

struct ConditionBuilderField(T, ModelFormat.Field field)
{
	// TODO: all the type specific field to Condition thingies

	private string columnName;

	this(string columnName) @safe
	{
		this.columnName = columnName;
	}

	private Condition* lhs() @safe
	{
		return makeConditionIdentifier(columnName);
	}

	Condition equals(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Equals, lhs, makeConditionConstant!field(value)));
	}

	Condition notEquals(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition lessThan(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Less, lhs, makeConditionConstant!field(value)));
	}

	Condition lessThanOrEqual(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.LessOrEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition greaterThan(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Greater, lhs, makeConditionConstant!field(value)));
	}

	Condition greaterThanOrEqual(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.GreaterOrEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition like(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Like, lhs, makeConditionConstant!field(value)));
	}

	Condition notLike(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotLike, lhs, makeConditionConstant!field(value)));
	}

	Condition regexp(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Regexp, lhs, makeConditionConstant!field(value)));
	}

	Condition notRegexp(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotRegexp, lhs, makeConditionConstant!field(value)));
	}

	Condition in_(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.In, lhs, makeConditionConstant!field(value)));
	}

	Condition notIn(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotIn, lhs, makeConditionConstant!field(value)));
	}

	Condition isNull() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNull, lhs));
	}

	alias equalsNull = isNull;

	Condition isNotNull() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNotNull, lhs));
	}

	alias notEqualsNull = isNotNull;

	Condition exists() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.Exists, lhs));
	}

	Condition notExists() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.NotExists, lhs));
	}

	Condition between(L, R)(L min, R max) @safe
	{
		return Condition(TernaryCondition(TernaryConditionType.Between, lhs, makeConditionConstant!field(min), makeConditionConstant!field(max)));
	}

	Condition notBetween(L, R)(L min, R max) @safe
	{
		return Condition(TernaryCondition(TernaryConditionType.NotBetween, lhs, makeConditionConstant!field(min), makeConditionConstant!field(max)));
	}
}

private struct JoinInformation
{
	private ffi.FFIJoin[] joins;
	/// Lookup foreign key name -> join placeholder
	private string[string] joinedTables;
}

struct SelectOperation(
	T,
	TSelect,
	bool hasWhere = false,
	bool hasOrder = false,
	bool hasOffset = false,
	bool hasLimit = false,
)
{
@safe:
	private const(DormDB)* db;
	private ffi.DBTransactionHandle tx;
	private ffi.FFICondition[] conditionTree;
	private JoinInformation joinInformation;
	private long offset, limit;

	static if (!hasWhere)
	{
		alias SelectBuilder = Condition delegate(ConditionBuilder!T);

		SelectOperation!(T, TSelect, true, hasOrder, hasLimit) condition(SelectBuilder callback) return @trusted
		{
			scope ConditionBuilderData data;
			scope ConditionBuilder!T builder;
			builder.builderData = &data;
			data.joinInformation = move(joinInformation);
			conditionTree = callback(builder).makeTree;
			joinInformation = move(data.joinInformation);
			return cast(typeof(return))this;
		}
	}

	static if (!hasOrder)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) orderBy(T...)(T) return @trusted
		{
			static assert(false, "not implemented");
		}
	}

	static if (!hasOffset)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) drop(long offset) return @trusted
		{
			this.offset = offset;
			return cast(typeof(return))this;
		}
	}

	static if (!hasLimit)
	{
		// may not .drop after take!
		SelectOperation!(T, TSelect, hasWhere, true, true) take(long limit) return @trusted
		{
			this.limit = limit;
			return cast(typeof(return))this;
		}
	}

	static if (!hasOffset && !hasLimit)
	{
		size_t[2] opSlice(size_t start, size_t end)
		{
			return [start, end];
		}

		SelectOperation!(T, TSelect, hasWhere, true, true) opIndex(size_t[2] slice) return @trusted
		{
			this.offset = slice[0];
			this.limit = cast(long)slice[1] - cast(long)slice[0];
			return cast(typeof(return))this;
		}
	}

	TSelect[] array() @trusted
	{
		enum fields = FilterLayoutFields!(T, TSelect);

		ffi.FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.columnName);

		mixin(makeRtColumns);

		TSelect[] ret;
		auto ctx = FreeableAsyncResult!(void delegate(scope ffi.FFIArray!(ffi.DBRowHandle))).make;
		ctx.forward_callback = (scope rows) {
			ret.length = rows.size;
			foreach (i; 0 .. rows.size)
				ret[i] = unwrapRowResult!(T, TSelect)(rows.data[i], joinInformation);
		};
		// TODO: pass in joins, limit, offset
		ffi.rorm_db_query_all(db.handle,
			tx,
			ffi.ffi(DormLayout!T.tableName),
			ffi.ffi(rtColumns),
			conditionTree.length ? &conditionTree[0] : null,
			ctx.callback.expand);
		ctx.result();
		return ret;
	}

	auto stream() @trusted
	{
		enum fields = FilterLayoutFields!(T, TSelect);

		ffi.FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.columnName);

		mixin(makeRtColumns);

		// TODO: pass in joins, limit, offset
		auto stream = sync_call!(ffi.rorm_db_query_stream)(db.handle,
			tx,
			ffi.ffi(DormLayout!T.tableName),
			ffi.ffi(rtColumns),
			conditionTree.length ? &conditionTree[0] : null);

		return RormStream!(T, TSelect)(stream, joinInformation);
	}

	TSelect findOne() @trusted
	{
		enum fields = FilterLayoutFields!(T, TSelect);

		ffi.FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.columnName);

		mixin(makeRtColumns);

		TSelect ret;
		auto ctx = FreeableAsyncResult!(void delegate(scope ffi.DBRowHandle)).make;
		ctx.forward_callback = (scope row) {
			ret = unwrapRowResult!(T, TSelect)(row, joinInformation);
		};
		// TODO: pass in joins, offset
		ffi.rorm_db_query_one(db.handle,
			tx,
			ffi.ffi(DormLayout!T.tableName),
			ffi.ffi(rtColumns),
			conditionTree.length ? &conditionTree[0] : null,
			ctx.callback.expand);
		ctx.result();
		return ret;
	}
}

private enum makeRtColumns = q{
	// inputs: ffi.FFIString[n] columns;
	//         JoinInformation joinInformation;
	//         T (template type)
	// output: ffi.FFIString[] rtColumns;

	ffi.FFIString[] rtColumns = columns[];
	if (joinInformation.joins.length)
	{
		static foreach (fk; DormForeignKeys!T)
		{{
			if (auto joinPrefix = fk.columnName in joinInformation.joinedTables)
			{
				enum filteredFields = FilterLayoutFields!(T, TSelect);
				size_t start = rtColumns.length;
				size_t i = 0;
				rtColumns.length += filteredFields.length;
				static foreach (field; filteredFields)
					rtColumns[start + (i++)] = ffi.ffi(*joinPrefix ~ ("." ~ field.columnName));
			}
		}}
	}
};

private struct RormStream(T, TSelect)
{
	import dorm.lib.util;

	private static struct RowHandleState
	{
		FreeableAsyncResult!(ffi.DBRowHandle) impl;
		alias impl this;
		bool done;

		void reset() @safe
		{
			impl.reset();
			done = false;
		}
	}

	extern(C) private static void rowCallback(void* data, ffi.DBRowHandle result, scope ffi.RormError error) nothrow @trusted
	{
		auto res = cast(RowHandleState*)data;
		if (error.tag == ffi.RormError.Tag.NoRowsLeftInStream)
			res.done = true;
		else if (error)
			res.error = error.makeException;
		else
			res.raw_result = result;
		res.event.set();
	}

	private ffi.DBStreamHandle handle;
	private RowHandleState currentHandle;
	private JoinInformation joinInformation;
	private bool started;

	this(ffi.DBStreamHandle handle, JoinInformation joinInformation = JoinInformation.init) @trusted
	{
		this.handle = handle;
		this.joinInformation = joinInformation;
		currentHandle = RowHandleState(FreeableAsyncResult!(ffi.DBRowHandle).make);
	}

	~this() @trusted
	{
		if (started)
		{
			currentHandle.impl.event.wait();
			if (currentHandle.impl.raw_result !is null)
				ffi.rorm_row_free(currentHandle.impl.raw_result);
			ffi.rorm_stream_free(handle);
		}
	}

	@disable this(this);

	int opApply(scope int delegate(TSelect) dg)
	{
		int result = 0;
		for (; !this.empty; this.popFront())
		{
			result = dg(this.front);
			if (result)
				break;
		}
		return result;
	}

	int opApply(scope int delegate(size_t i, TSelect) dg)
	{
		int result = 0;
		size_t i;
		for (; !this.empty; this.popFront())
		{
			result = dg(i++, this.front);
			if (result)
				break;
		}
		return result;
	}

	auto front() @property @trusted
	{
		if (!started) nextIteration();
		return unwrapRowResult!(T, TSelect)(currentHandle.result(), joinInformation);
	}
	
	bool empty() @property @trusted
	{
		if (!started) nextIteration();
		currentHandle.impl.event.wait();
		return currentHandle.done;
	}

	void popFront() @trusted
	{
		if (!started) nextIteration();
		currentHandle.impl.event.wait();
		if (currentHandle.done)
			throw new Exception("attempted to run popFront on ended stream");
		else if (currentHandle.impl.error)
			throw currentHandle.impl.error;
		else
		{
			ffi.rorm_row_free(currentHandle.impl.raw_result);
			currentHandle.reset();
			nextIteration();
		}
	}

	private void nextIteration() @trusted
	{
		started = true;
		ffi.rorm_stream_get_row(handle, &rowCallback, cast(void*)&currentHandle);
	}

	static assert(isInputRange!RormStream, "implementation error: did not become an input range");
}

TSelect unwrapRowResult(T, TSelect)(ffi.DBRowHandle row, JoinInformation ji) @safe
{
	auto base = unwrapRowResultImpl!(T, TSelect, false)(row, null);
	if (ji.joins.length)
	{
		static foreach (fk; DormForeignKeys!T)
		{{
			if (auto pprefix = fk.columnName in ji.joinedTables)
			{
				string prefix = *pprefix;
				alias ModelRef = typeof(__traits(getMember, T, fk.sourceColumn));
				__traits(getMember, base, fk.sourceColumn) =
					unwrapRowResult!(ModelRef.TModel, ModelRef.TSelect)(row, prefix);
			}
		}}
	}
	return base;
}

TSelect unwrapRowResult(T, TSelect)(ffi.DBRowHandle row) @safe
{
	return unwrapRowResultImpl!(T, TSelect, false)(row, null);
}

TSelect unwrapRowResult(T, TSelect)(ffi.DBRowHandle row, string placeholder) @safe
{
	scope placeholderDot = new char[placeholder.length + 1];
	placeholderDot[0 .. placeholder.length] = placeholder;
	placeholderDot[$ - 1] = '.';
	return unwrapRowResultImpl!(T, TSelect, true)(row, (() @trusted => cast(string)placeholderDot)());
}

private TSelect unwrapRowResultImpl(T, TSelect, bool usePrefix)(ffi.DBRowHandle row, string prefixWithDot) @safe
{
	TSelect res;
	static if (is(TSelect == class))
		res = new TSelect();
	ffi.RormError rowError;
	enum fields = FilterLayoutFields!(T, TSelect);
	static foreach (field; fields)
		mixin("res." ~ field.sourceColumn) = extractField!(field, typeof(mixin("res." ~ field.sourceColumn)),
			text(" from model ", T.stringof, " in column ", field.sourceColumn, " in file ", field.definedAt).idup,
			usePrefix)(row, rowError, prefixWithDot);
	if (rowError)
			throw rowError.makeException(" (in column '" ~ columnPrefix ~ field.columnName ~ "')");
	return res;
}

private T extractField(alias field, T, string errInfo, bool usePrefix)(ffi.DBRowHandle row, ref ffi.RormError error, string prefixWithDot) @trusted
{
	import std.conv;
	import dorm.declarative;

	static if (usePrefix)
		auto columnName = ffi.ffi(prefixWithDot ~ field.columnName);
	else
		auto columnName = ffi.ffi(field.columnName);

	enum pre = field.isNullable() ? "ffi.rorm_row_get_null_" : "ffi.rorm_row_get_";
	enum suf = "(row, columnName, error)";

	final switch (field.type) with (ModelFormat.Field.DBType)
	{
		case varchar:
			static if (field.type == varchar) return fieldInto!(T, errInfo)(mixin(pre, "str", suf), error);
			else assert(false);
		case varbinary:
			static if (field.type == varbinary) return fieldInto!(T, errInfo)(mixin(pre, "binary", suf), error);
			else assert(false);
		case int8:
			static if (field.type == int8) return fieldInto!(T, errInfo)(mixin(pre, "i16", suf), error);
			else assert(false);
		case int16:
			static if (field.type == int16) return fieldInto!(T, errInfo)(mixin(pre, "i16", suf), error);
			else assert(false);
		case int32:
			static if (field.type == int32) return fieldInto!(T, errInfo)(mixin(pre, "i32", suf), error);
			else assert(false);
		case int64:
			static if (field.type == int64) return fieldInto!(T, errInfo)(mixin(pre, "i64", suf), error);
			else assert(false);
		case floatNumber:
			static if (field.type == floatNumber) return fieldInto!(T, errInfo)(mixin(pre, "f32", suf), error);
			else assert(false);
		case doubleNumber:
			static if (field.type == doubleNumber) return fieldInto!(T, errInfo)(mixin(pre, "f64", suf), error);
			else assert(false);
		case boolean:
			static if (field.type == boolean) return fieldInto!(T, errInfo)(mixin(pre, "bool", suf), error);
			else assert(false);
		case date:
			static if (field.type == date) return fieldInto!(T, errInfo)(mixin(pre, "date", suf), error);
			else assert(false);
		case time:
			static if (field.type == time) return fieldInto!(T, errInfo)(mixin(pre, "time", suf), error);
			else assert(false);
		case datetime:
			static if (field.type == datetime) return fieldInto!(T, errInfo)(mixin(pre, "datetime", suf), error);
			else assert(false);

		static assert(
			field.type != set,
			"field type " ~ field.type.to!string ~ " not yet implemented for reading");

		case choices:
			static if (field.type == choices) return fieldInto!(T, errInfo)(mixin(pre, "str", suf), error);
			else assert(false);
		case set: assert(false);
	}
}

private T fieldInto(T, string errInfo, From)(scope From v, ref ffi.RormError error) @safe
{
	import dorm.lib.ffi : FFIArray, FFIOption;
	import std.typecons : Nullable;

	static if (is(T == From))
		return v;
	else static if (is(T == enum))
	{
		auto s = fieldInto!(string, errInfo, From)(v, error);
		static if (is(OriginalType!T == string))
			return cast(T)s;
		else
		{
			switch (s)
			{
				static foreach (f; __traits(allMembers, T))
				{
				case f:
					return __traits(getMember, T, f);
				}
				default:
					error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
					return T.init;
			}
		}
	}
	else static if (is(T == ModelRefImpl!(id, _TModel, _TSelect), alias id, _TModel, _TSelect))
	{
		T ret;
		ret.foreignKey = fieldInto!(typeof(id), errInfo, From)(v, error);
		return ret;
	}
	else static if (is(From == FFIArray!U, U))
	{
		static if (is(T == Res[], Res))
		{
			static if (is(immutable Res == immutable U))
				return (() @trusted => cast(T)v.data.dup)();
			else
				static assert(false, "can't auto-wrap array element type " ~ Res.stringof ~ " into " ~ U.stringof ~ errInfo);
		}
		else static if (is(T == Nullable!V, V))
		{
			return T(fieldInto!(V, errInfo, From)(v, error));
		}
		else
			static assert(false, "can't auto-wrap " ~ U.stringof ~ "[] into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From == FFIOption!U, U))
	{
		static if (is(T == Nullable!V, V))
		{
			if (v.isNull)
				return T.init;
			else
				return T(fieldInto!(V, errInfo)(v.raw_value, error));
		}
		else static if (__traits(compiles, T(null)))
		{
			if (v.isNull)
				return T(null);
			else
				return fieldInto!(T, errInfo)(v.raw_value, error);
		}
		else
		{
			if (v.isNull)
			{
				error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
				return T.init;
			}
			else
			{
				return fieldInto!(T, errInfo)(v.raw_value, error);
			}
		}
	}
	else static if (is(T == Nullable!U, U))
	{
		return T(fieldInto!(U, errInfo, From)(v, error));
	}
	else static if (isIntegral!From)
	{
		static if (isIntegral!T && From.sizeof >= T.sizeof)
		{
			if (v < cast(From)T.min || v > cast(From)T.max)
			{
				error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
				return T.init;
			}
			else
			{
				return cast(T)v;
			}
		}
		else static if (isFloatingPoint!T)
		{
			return cast(T)v;
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (isFloatingPoint!From)
	{
		static if (isFloatingPoint!T)
			return cast(T)v;
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From : ffi.FFITime))
	{
		static if (is(T == TimeOfDay))
		{
			try
			{
				return TimeOfDay(cast(int)v.hour, cast(int)v.min, cast(int)v.sec);
			}
			catch (DateTimeException)
			{
				error = ffi.RormError(ffi.RormError.Tag.InvalidTimeError);
				return T.init;
			}
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From : ffi.FFIDate))
	{
		static if (is(T == Date))
		{
			try
			{
				return Date(cast(int)v.year, cast(int)v.month, cast(int)v.day);
			}
			catch (DateTimeException)
			{
				error = ffi.RormError(ffi.RormError.Tag.InvalidDateError);
				return T.init;
			}
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From : ffi.FFIDateTime))
	{
		try
		{
			static if (is(T == DateTime))
			{
				return DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec);
			}
			else static if (is(T == SysTime))
			{
				return SysTime(DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec), UTC());
			}
			else static if (is(T == long) || is(T == ulong))
			{
				return cast(T)SysTime(DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec), UTC()).stdTime;
			}
			else
				static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
		}
		catch (DateTimeException)
		{
			error = ffi.RormError(ffi.RormError.Tag.InvalidDateTimeError);
			return T.init;
		}
	}
	else
		static assert(false, "did not implement conversion from " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
}

mixin template SetupDormRuntime(alias timeout = 10.seconds)
{
	__gshared bool _initializedDormRuntime;

	shared static this()
	{
		import dorm.lib.util : sync_call;
		import dorm.lib.ffi : rorm_runtime_start;

		sync_call!(rorm_runtime_start)();
		_initializedDormRuntime = true;
	}

	shared static ~this()
	{
		import core.time : Duration;
		import dorm.lib.util;
		import dorm.lib.ffi : rorm_runtime_shutdown;

		if (_initializedDormRuntime)
		{
			static if (is(typeof(timeout) == Duration))
				sync_call!(rorm_runtime_shutdown)(timeout.total!"msecs");
			else
				sync_call!(rorm_runtime_shutdown)(timeout);
		}
	}
}
