# Changelog

## 0.1.0

First release of this community SDK.

- Load Lore's C library and reject any interface version other than 0.10.1.
- Local repository create, status, stage, unstage, commit, history, diff,
  branch list, branch create, and branch switch.
- Write a committed file to disk, or read it with an explicit size limit.
- Discover a repository by walking parents for a `.lore` directory.

Remote operations, authentication, merges, locks, and the storage streaming
API are not wrapped.
