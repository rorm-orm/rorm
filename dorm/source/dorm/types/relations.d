module dorm.types.relations;

import dorm.declarative.conversion;
import dorm.model;
import dorm.types.patches;

version(none) static struct ManyToManyField(alias idOrModel)
{
	alias T = ModelFromIdOrModel!idOrModel;
	alias primaryKeyAlias = IdAliasFromIdOrModel!idOrModel;
	enum primaryKeyField = IdFieldFromIdOrModel!idOrModel;
	alias PrimaryKeyType = typeof(primaryKeyAlias);

	bool toClear;
	PrimaryKeyType[] toAdd;
	PrimaryKeyType[] toRemove;

	private T[] cached;
	private bool resolved;

	T[] populated()
	{
		assert(resolved, "ManyToManyField reference is not populated! Call "
			~ "`db.populate!(Model.manyToManyFieldName)(modelInstance)` or query "
			~ "data with the recursion flag set!");
		return cached;
	}

	void setCachedPopulated(T[] populated)
	{
		cached = populated;
		resolved = true;
	}

	void add(T other)
	{
		auto refField = __traits(child, other, primaryKeyAlias);
		toRemove = toRemove.remove!(refField);
		toAdd ~= refField;
	}

	void add(PrimaryKeyType primaryKey)
	{
		toRemove = toRemove.remove!(primaryKey);
		toAdd ~= primaryKey;
	}

	void add(Range)(Range range)
	if (!is(Range == T)
	&& !is(Range == PrimaryKeyType))
	{
		foreach (item; range)
			add(item);
	}

	void remove(T other)
	{
		auto refField = __traits(child, other, primaryKeyAlias);
		toAdd = toAdd.remove!(refField);
		toRemove ~= refField;
	}

	void add(PrimaryKeyType primaryKey)
	{
		toRemove = toRemove.remove!(primaryKey);
		toAdd ~= primaryKey;
	}

	void remove(Range)(Range range)
	if (!is(Range == T)
	&& !is(Range == PrimaryKeyType))
	{
		foreach (item; range)
			remove(item);
	}

	void clear()
	{
		toAdd.length = 0;
		toRemove.length = 0;
		toClear = true;
	}
}

static template ModelRef(alias idOrPatch)
{
	alias primaryKeyAlias = IdAliasFromIdOrPatch!idOrPatch;
	alias TPatch = PatchFromIdOrPatch!idOrPatch;
	alias T = ModelFromSomePatch!TPatch;
	alias ModelRef = ModelRefImpl!(primaryKeyAlias, T, TPatch);
}

static struct ModelRefImpl(alias id, _TModel, _TSelect)
{
	alias TModel = _TModel;
	alias TSelect = _TSelect;
	alias primaryKeyAlias = id;
	enum primaryKeyField = DormField!(_TModel, __traits(identifier, id));
	alias PrimaryKeyType = typeof(primaryKeyAlias);

	PrimaryKeyType foreignKey;

	private TSelect cached;
	private bool resolved;

	TSelect populated()
	{
		assert(resolved, "ModelRef reference is not populated! Call "
			~ "`db.populate!(Model.referenceFieldName)(modelInstance)` or query "
			~ "data with the recursion flag set!");
		return cached;
	}

	auto opAssign(TSelect value)
	{
		resolved = true;
		cached = value;
		foreignKey = __traits(child, value, primaryKeyAlias);
		return value;
	}
}

static struct BackRef(alias foreignField)
{
	static assert(is(__traits(parent, foreignField) : Model),
		"Invalid foreign key field `" ~ foreignField.stringof
		~ "`! Change to `BackRef!(OtherModel.foreignKeyReferencingThis)`");

	alias T = __traits(parent, foreignField);

	private T[] cached;
	private bool resolved;

	T[] populated()
	{
		assert(resolved, "BackRef value is not populated! Call "
			~ "`db.populate!(Model.otherFieldReferencingThis)(modelInstance)` or query "
			~ "data with the recursion flag set!");
		return cached;
	}
}
