module dorm.lib.ffi_wrap;

// public import dorm.lib.ffi_impl;

import std.array;
import std.conv;
import std.stdio;
import std.traits;
import std.typecons : tuple;

private static import dorm.lib.ffi_impl;

private string formatArgs(T...)(T args)
{
	auto ret = appender!string;
	bool first = true;
	static foreach (arg; args)
	{
		if (!first)
			ret ~= ", ";
		first = false;
		ret ~= "\n\t";
		ret ~= formatArg(arg);
	}
	static if (args.length > 0)
		ret ~= "\n";
	return ret.data;
}

private string formatArg(T)(T value)
{
	static if (is(T : const(FFICondition)*) || is(T : FFICondition*))
	{
		return value ? formatArg(*value) : "(no condition)";
	}
	else static if (is(T == enum))
	{
		static if (is(OriginalType!T == U*, U))
			return T.stringof ~ "@" ~ (cast(size_t)value).to!string(16);
		else
			return T.stringof ~ "." ~ value.to!string;
	}
	else static if (is(T == U*, U))
		return U.stringof ~ "@" ~ (cast(size_t)value).to!string(16);
	else
	{
		return value.to!string;
	}
}

size_t dormTraceFun(T...)(bool withRet, string method, T args)
{
	static __gshared size_t id;
	if (withRet)
	{
		auto reqId = id++;
		stderr.writeln("[trace] #", reqId, " ", method, " (", formatArgs(args), ")");
		return reqId;
	}
	else
	{
		stderr.writeln("[trace] (void) ", method, "(", formatArgs(args), ")");
		return 0;
	}
}

void dormTraceRetval(T)(size_t reqid, T retval)
{
	stderr.writeln("[trace] #", reqid, " -> returned ", retval);
}

void dormTraceCallback(scope dorm.lib.ffi_impl.RormError error)
{
	if (error)
		stderr.writeln("[trace] Callback (failed) -> ", error.makeException.msg);
	else
		stderr.writeln("[trace] Callback success");
}

void dormTraceCallback(T)(scope T result, scope dorm.lib.ffi_impl.RormError error)
{
	if (error)
		stderr.writeln("[trace] Callback (failed) -> ", error.makeException.msg);
	else
		stderr.writeln("[trace] Callback success: ", formatArg(result));
}

void dormTraceSyncCallback(scope dorm.lib.ffi_impl.RormError error)
{
	if (error)
		stderr.writeln("[trace] Sync Callback (failed) -> ", error.makeException.msg);
	else
		stderr.writeln("[trace] Sync Callback success");
}

void dormTraceSyncCallback(T)(scope T result, scope dorm.lib.ffi_impl.RormError error)
{
	if (error)
		stderr.writeln("[trace] Sync Callback (failed) -> ", error.makeException.msg);
	else
		stderr.writeln("[trace] Sync Callback success: ", formatArg(result));
}

// ---------------------------------------------------------------------------------

private static string generateTracer(string symbolName)()
{
	alias symbol = __traits(getMember, dorm.lib.ffi_impl, symbolName);

	static if (is(ReturnType!symbol == void))
	{
		return "extern(D) " ~ ReturnType!symbol.stringof
			~ " " ~ symbolName ~ "_wrapper(Parameters!(dorm.lib.ffi_impl." ~ symbolName ~ ")) {
				import dorm.lib.ffi_impl : " ~ symbolName ~ ";
				dormTraceFun(false, `" ~ symbolName ~ "`, __traits(parameters));
				" ~ symbolName ~ "(__traits(parameters));
			}
			alias " ~ symbolName ~ " = " ~ symbolName ~ "_wrapper;";
	}
	else
	{
		return "extern(D) " ~ ReturnType!symbol.stringof
			~ " " ~ symbolName ~ "_wrapper(Parameters!(dorm.lib.ffi_impl." ~ symbolName ~ ")) {
				import dorm.lib.ffi_impl : " ~ symbolName ~ ";
				auto reqid = dormTraceFun(true, `" ~ symbolName ~ "`, __traits(parameters));

				auto retval = " ~ symbolName ~ "(__traits(parameters));
				dormTraceRetval(reqid, retval);
				return retval;
			}
			alias " ~ symbolName ~ " = " ~ symbolName ~ "_wrapper;";
	}
}

static foreach (symbol; __traits(allMembers, dorm.lib.ffi_impl))
{
	static if (__traits(compiles, mixin("dorm.lib.ffi_impl.", symbol)))
	{
		static if (is(typeof(__traits(getMember, dorm.lib.ffi_impl, symbol)) == function)
			&& __traits(getLinkage, __traits(getMember, dorm.lib.ffi_impl, symbol)) == "C")
		{
			mixin(generateTracer!symbol);
		}
		else static if (symbol != "object" && symbol != "dorm")
			mixin("alias ", symbol , " = dorm.lib.ffi_impl.", symbol , ";");
	}
}

