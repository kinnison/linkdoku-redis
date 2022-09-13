-- Creating a role in the Linkdoku Redis
--
-- Script must be called with the following keys:
--   role:{uuid}
--   role:byname
--   identity:{owneruuid}:roles
-- And the following arguments are expected, in the following order
--   uuid
--   owner
--   short_name
--   display_name
--   bio
--
-- If the given short_name is already in use, this script *will* error
-- otherwise it will create the role and also set the short name for the
-- role to be reserved

local role_key, role_byname, owner_roles = KEYS[1], KEYS[2], KEYS[3]
local uuid, owner, short_name, display_name, bio = ARGV[1], ARGV[2], ARGV[3], ARGV[4], ARGV[5]

-- First we try and retrieve a role by the short name

local byname = redis.call("HEXISTS", role_byname, short_name)
if byname == 1 then
    return redis.error_reply("short-name-exists")
end

-- OK, we should be able to insert so let's do that
redis.call("HSET", role_byname, short_name, uuid)
redis.call("SADD", owner_roles, uuid)
return redis.pcall("HSET", role_key, "owner", owner, "short_name", short_name, "display_name", display_name, "bio", bio)
