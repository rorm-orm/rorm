module models;

import dorm.design;

mixin RegisterModels;

class User : Model
{
	@maxLength(255) @primaryKey
	string username;
	@maxLength(255)
	string fullName;
	@maxLength(255)
	string email;
}

class Toot : Model
{
	@maxLength(2048)
	string message;
	@autoCreateTime
	SysTime createdAt;
	ModelRef!User author;
}
