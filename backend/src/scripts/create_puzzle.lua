-- Creating a puzzle in the Linkdoku Redis
--
-- Script must be called with the following keys:
--   puzzle:{uuid}
--   puzzle:byname
--   role:{owneruuid}:puzzles
-- And the following arguments are expected, in the following order
--   uuid
--   owner
--   short_name
--   display_name
--   visibility
--   visibility_date
--   states
--
-- If the given short_name is already in use, this script *will* error
-- otherwise it will create the role and also set the short name for the
-- role to be reserved

local puzzle_key, puzzle_byname, owner_puzzles = KEYS[1], KEYS[2], KEYS[3]
local uuid, owner, short_name, display_name, visibility, visibility_date, states = ARGV[1], ARGV[2], ARGV[3], ARGV[4], ARGV[5], ARGV[6], ARV[7]

-- First we try and retrieve a puzzle by the short name

local byname = redis.call("HEXISTS", puzzle_byname, short_name)
if byname == 1 then
    return redis.error_reply("short-name-exists")
end

-- OK, we should be able to insert so let's do that
redis.call("HSET", puzzle_byname, short_name, uuid)
redis.call("SADD", owner_puzzles, uuid)
return redis.pcall("HSET", puzzle_key, "owner", owner, "short_name", short_name, "display_name", display_name, "visibility", visibility, "visibility_date", visibility_date, "states", states)
