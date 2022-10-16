module models;

import dorm.design;

mixin RegisterModels;

enum State : string
{
	ok = "OK",
	warn = "WARN",
	critical = "CRIT",
	unknown = "UNKN"
}

abstract class NamedThing : Model
{
	@maxLength(255)
	string name;
}

struct SuperCommon
{
	int superCommonField;
}

struct Common
{
	string commonName;
	@embedded
	SuperCommon superCommon;
}

class User : NamedThing
{
	@maxLength(255)
	string password;

	@maxLength(255)
	Nullable!string email;

	ubyte age;

	Nullable!DateTime birthday;

	@autoCreateTime
	SysTime createdAt;

	@autoUpdateTime
	Nullable!SysTime updatedAt;

	@autoCreateTime
	ulong createdAt2;

	@autoUpdateTime
	Nullable!ulong updatedAt2;

	State state;

	@choices("ok", "warn", "critical", "unknown")
	string state2;

	@columnName("admin")
	bool isAdmin;

	@constructValue!(() => Clock.currTime + 4.hours)
	@autoCreateTime
	SysTime validUntil;

	@maxLength(255)
	@defaultValue("")
	string comment;

	@defaultValue(1337)
	int counter;

	@primaryKey
	long ownPrimaryKey;

	@timestamp
	Nullable!ulong someTimestamp;

	@unique
	int uuid;

	@validator!(x => x >= 18)
	int someInt;

	@ignored
	int imNotIncluded;

	@embedded
	Common commonFields;
}
