# Linkdoku

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

# Developing linkdoku

To develop linkdoku, we have a `docker-compose.yml` which will stand up a redis and a docker capable
of building and running linkdoku. Note, this is not a release-mode docker, but a development docker.

The development docker also uses `cargo watch` so once it's up and running you can edit source and save
it and it should rebuild the frontend and backend automagically.

It won't always quit on first `^C` - if it gets stuck, hit that again.
