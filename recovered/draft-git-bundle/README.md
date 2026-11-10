# Historical draft Git bundle

This incremental bundle preserves eight exact draft commits. It requires the
recorded base commit in a source Git repository; it does not assert that the
drafts are reviewed, integrated, current or validated.

Run `python restore_drafts.py SOURCE_REPOSITORY NEW_DESTINATION` to create a
separate Git repository using the local source repository as an object
alternate, verify the bundle and restore the eight pinned refs. The destination must not
exist. No branch is checked out, and the source repository is not modified.
The restored repository depends on the source repository’s object store; keep
that store available or make an independent copy before removing it. An
incomplete marker remains if restoration fails. If the base commit is not
available locally, obtain it through normal repository access first.
