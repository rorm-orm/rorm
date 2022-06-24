/// This module converts a package containing D Model definitions to
/// $(REF SerializedModels, dorm,declarative)
module dorm.declarative.conversion;

version (unittest) import dorm.model;

SerializedModels processModelsToDeclarations(alias mod)()
{
	SerializedModels ret;
	return ret;
}

unittest
{
	struct Mod
	{
		enum State : string
		{
			ok = "ok",
			warn = "warn",
			critical = "critical",
			unknown = "unknown"
		}

		class User : Model
		{
			@maxLength(255)
			string username;

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
			SysTime validUntil;

			@maxLength(255)
			@defaultValue("")
			string comment;

			@defaultValue(1337)
			int counter;

			@primaryKey
			ulong ownPrimaryKey;

			@timestamp
			ulong creationTime;

			@unique
			int uuid;

			@validator!(x => x => 18)
			int someInt;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.validators.length == 1);
	assert(mod.valueConstructors.length == 1);
	assert(mod.models.length == 1);

	auto validatorFunId = mod.validators.front.key;
	auto constructFunId = mod.valueConstructors.front.key;

	auto m = mod.models[0];
	assert(m.fields.length == 22);

	assert(m.fields[0].name == "username");
	assert(m.fields[0].type == ModelFormat.Field.DBType.varchar);
	assert(!m.fields[0].nullable);
	assert(m.fields[0].annotations == [SerializedAnnotation(maxLength(255))]);

	assert(m.fields[1].name == "password");
	assert(m.fields[1].type == ModelFormat.Field.DBType.varchar);
	assert(!m.fields[1].nullable);
	assert(m.fields[1].annotations == [SerializedAnnotation(maxLength(255))]);

	assert(m.fields[2].name == "email");
	assert(m.fields[2].type == ModelFormat.Field.DBType.varchar);
	assert(m.fields[2].nullable);
	assert(m.fields[2].annotations == [SerializedAnnotation(maxLength(255))]);

	assert(m.fields[3].name == "age");
	assert(m.fields[3].type == ModelFormat.Field.DBType.uint8);
	assert(!m.fields[3].nullable);
	assert(m.fields[3].annotations == []);

	assert(m.fields[4].name == "birthday");
	assert(m.fields[4].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[4].nullable);
	assert(m.fields[4].annotations == []);

	assert(m.fields[5].name == "created_at");
	assert(m.fields[5].type == ModelFormat.Field.DBType.timestamp);
	assert(!m.fields[5].nullable);
	assert(m.fields[5].annotations == [AnnotationFlag.autoCreateTime]);

	assert(m.fields[6].name == "updated_at");
	assert(m.fields[6].type == ModelFormat.Field.DBType.timestamp);
	assert(m.fields[6].nullable);
	assert(m.fields[6].annotations == [AnnotationFlag.autoUpdateTime]);

	assert(m.fields[7].name == "created_at_2");
	assert(m.fields[7].type == ModelFormat.Field.DBType.timestamp);
	assert(!m.fields[7].nullable);
	assert(m.fields[7].annotations == [AnnotationFlag.autoCreateTime]);

	assert(m.fields[8].name == "updated_at_2");
	assert(m.fields[8].type == ModelFormat.Field.DBType.timestamp);
	assert(m.fields[8].nullable);
	assert(m.fields[8].annotations == [AnnotationFlag.autoUpdateTime]);

	assert(m.fields[9].name == "state");
	assert(m.fields[9].type == ModelFormat.Field.DBType.choices);
	assert(!m.fields[9].nullable);
	assert(m.fields[9].annotations == [Choices(["ok", "warn", "critical", "unknown"])]);

	assert(m.fields[10].name == "state_2");
	assert(m.fields[10].type == ModelFormat.Field.DBType.choices);
	assert(!m.fields[10].nullable);
	assert(m.fields[10].annotations == [Choices(["ok", "warn", "critical", "unknown"])]);

	assert(m.fields[11].name == "admin");
	assert(m.fields[11].type == ModelFormat.Field.DBType.boolean);
	assert(!m.fields[11].nullable);
	assert(m.fields[11].annotations == []);

	assert(m.fields[12].name == "valid_until");
	assert(m.fields[12].type == ModelFormat.Field.DBType.timestamp);
	assert(!m.fields[12].nullable);
	assert(m.fields[12].annotations == [ConstructValueRef(constructFunId)]);

	assert(m.fields[13].name == "comment");
	assert(m.fields[13].type == ModelFormat.Field.DBType.varchar);
	assert(!m.fields[13].nullable);
	assert(m.fields[13].annotations == [maxLength(255), defaultValue("")]);

	assert(m.fields[14].name == "counter");
	assert(m.fields[14].type == ModelFormat.Field.DBType.int32);
	assert(!m.fields[14].nullable);
	assert(m.fields[14].annotations == [defaultValue(1337)]);

	assert(m.fields[15].name == "own_primary_key");
	assert(m.fields[15].type == ModelFormat.Field.DBType.uint64);
	assert(!m.fields[15].nullable);
	assert(m.fields[15].annotations == [AnnotationFlag.primaryKey]);

	assert(m.fields[16].name == "creation_time");
	assert(m.fields[16].type == ModelFormat.Field.DBType.timestamp);
	assert(!m.fields[16].nullable);
	assert(m.fields[16].annotations == []);

	assert(m.fields[17].name == "uuid");
	assert(m.fields[17].type == ModelFormat.Field.DBType.int32);
	assert(!m.fields[17].nullable);
	assert(m.fields[17].annotations == [AnnotationFlag.unique]);

	assert(m.fields[18].name == "some_int");
	assert(m.fields[18].type == ModelFormat.Field.DBType.int32);
	assert(!m.fields[18].nullable);
	assert(m.fields[18].annotations == [ValidatorRef(validatorFunId)]);
}
