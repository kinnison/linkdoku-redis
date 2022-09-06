-- Upserting an identity in the Linkdoku redis
-- Script must be invoked with the following keys:
--   identity:UUID
--   identity:UUID:roles
-- Also the following arguments are expected, in order:
--   display_name
--   gravatar_hash (empty string if unavailable)
-- The script will upsert the identity and return the roles
-- that the identity has (empty list on new identity)

local key_id, key_roles = KEYS[1], KEYS[2]
local display_name, gravatar_hash = ARGV[1], ARGV[2]

if gravatar_hash ~= "" then
    redis.call("HSET", key_id, "display_name", display_name, "gravatar_hash", gravatar_hash)
else
    redis.call("HSET", key_id, "display_name", display_name)
end

return redis.pcall("LRANGE", key_roles, 0, -1)
