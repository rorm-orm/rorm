module models;

import dorm.design;

mixin RegisterModels;

class User : Model
{
	@maxLength(255)
	string username;

	@maxLength(255)
	string password;

	@maxLength(255)
	Nullable!string email;

	@autoCreateTime
	SysTime createdAt;

	@autoUpdateTime
	Nullable!SysTime updatedAt;

	@columnName("admin")
	bool isAdmin;

	@constructValue!(() => Clock.currTime + 24.hours)
	SysTime tempPasswordTime;
}
