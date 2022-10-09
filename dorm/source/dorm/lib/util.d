module dorm.lib.util;

import core.sync.event;
import std.algorithm;
import std.functional;
import std.traits;
import std.typecons;

import dorm.lib.ffi;

struct FreeableAsyncResult(T)
{
	Event event;
	static if (!is(T == void))
		T raw_result;
	Exception error;

	@disable this();

	this(Event event) @trusted
	{
		this.event = move(event);
	}

	static FreeableAsyncResult make() @trusted
	{
		return FreeableAsyncResult(Event(true, false));
	}

	static if (is(T == void))
		alias Callback = extern(C) void function(void* data, scope RormError error) nothrow;
	else
		alias Callback = extern(C) void function(void* data, T result, scope RormError error) nothrow;

	Tuple!(Callback, void*) callback() return @safe
	{
		static if (is(T == void))
		{
			extern(C) static void ret(void* data, scope RormError error) nothrow
			{
				auto res = cast(FreeableAsyncResult*)data;
				if (error)
					res.error = error.makeException;
				res.event.set();
			}
		}
		else
		{
			extern(C) static void ret(void* data, T result, scope RormError error) nothrow
			{
				auto res = cast(FreeableAsyncResult*)data;
				if (error)
					res.error = error.makeException;
				else
					res.raw_result = result;
				res.event.set();
			}
		}

		return tuple(&ret, cast(void*)&this);
	}

	T result() @safe
	{
		(() @trusted => event.wait())();
		if (error)
			throw error;
		static if (!is(T == void))
			return raw_result;
	}

	void reset() @safe
	{
		(() @trusted => event.reset())();
		static if (!is(T == void))
			raw_result = T.init;
		error = null;
	}
}

auto sync_call(alias fn)(Parameters!fn[0 .. $ - 2] args) @trusted
{
	static assert(Parameters!(Parameters!fn[$ - 2]).length == 3
		|| Parameters!(Parameters!fn[$ - 2]).length == 2);
	static assert(is(Parameters!(Parameters!fn[$ - 2])[0] == void*));
	static assert(is(Parameters!(Parameters!fn[$ - 2])[$ - 1] == RormError));

	enum isVoid = Parameters!(Parameters!fn[$ - 2]).length == 2;

	struct Result
	{
		Exception exception;
		static if (!isVoid)
			Parameters!(Parameters!fn[$ - 2])[1] ret;
		bool sync;
	}

	Result result;

	extern(C) static void callback(Parameters!(Parameters!fn[$ - 2]) args) nothrow
	{
		auto result = cast(Result*)(args[0]);
		static if (!isVoid)
			auto data = args[1];
		auto error = args[$ - 1];
		if (error) result.exception = error.makeException;
		else {
			static if (!isVoid)
				result.ret = data;
		}
		result.sync = true;
	}
	fn(forward!args, &callback, &result);
	assert(result.sync, "called sync_call with function that does not call its callback in synchronous context!");

	if (result.exception)
		throw result.exception;

	static if (!isVoid)
		return result.ret;
}

template ffiInto(To)
{
	To ffiInto(From)(From v)
	{
		static assert(From.tupleof.length == To.tupleof.length,
			"FFI member fields count mismatch between "
			~ From.stringof ~ " and " ~ To.stringof);

		To ret;
		foreach (i, ref field; ret.tupleof)
		{
			static if (is(typeof(field) == FFIArray!T, T))
				field = FFIArray!T.fromData(v.tupleof[i]);
			else
				field = v.tupleof[i];
		}
		return ret;
	}
}
