# Filesystem contract

`EntryName` identifies one child in a directory namespace. Construct it at the
path-splitting or external-name admission boundary. It rejects empty names,
`.` and `..`, either path separator, a Windows drive prefix, and NUL. Accepted
Unicode and spelling are retained exactly; other platform-specific filename
restrictions remain provider errors. Ordinary colon-containing names remain
accepted. A child name does not prove existence, permission, or file kind.

`DirectoryEntry::get_file` and `get_directory` take `EntryName` and
`EntryLookupIntent`. `Existing` performs lookup without creating a child.
`CreateIfMissing` permits creation and preserves existing content. Native file
creation is exclusive to avoid truncating a child created concurrently.
`delete_entry` also takes an admitted `EntryName`.

Native namespaces are admitted from `PathBuf`. Native lookup without creation
retains lazy path references, so a missing child can fail on a subsequent read.
Browser lookup follows File System Access handle semantics and can reject
missing or inaccessible children immediately. Neither backend's address or
handle bypasses its provider's access decisions.

`EntryLabel` is a provider display label, distinct from the admitted
`EntryName` used for child lookup. It retains native filename bytes and converts
to Unicode only when formatted for presentation. Empty labels are valid for
unnamed roots. Labels cannot serve as serialized resource addresses.

`DirectoryNamespace` retains the concrete native directory or browser handle.
Browser handles do not supply a filesystem path. File and directory entries
therefore expose their display label and operations; directories also expose
their namespace authority.

`ProjectRoot` owns the selected project namespace, activation, and persistence.
Selection does not bypass access checks. `current`, `activate`, and `clear`
manage the process's selected root. Project resource resolution retains that
owner and checks native namespace components after URI decoding. Browser
project resources use the explicit `prism://project/` namespace.

Native persistence writes one absolute root to the application-owned
`workspace.path` record. Unix records retain exact OS filename bytes; Windows
records retain little-endian OS UTF-16 code units. No trimming or URI rewriting
occurs. Empty, NUL-containing, relative, or incomplete Windows records fail
admission with the rejected `FileContents`. Saving validates a real directory.
Restoration distinguishes no record, a missing saved root, and a restored root;
an existing non-directory target is an error.

Browser persistence uses IndexedDB's structured cloning for the selected
directory handle. It retains database exceptions and rejects malformed stored
values. `WorkspaceStore` owns the database name/version, handle store, and root
key at the storage boundary. Each operation retains database, transaction, and
request listener owners until it settles. Reads and writes wait for transaction
completion; a request's success cannot hide a subsequent abort. Listeners detach
before their callbacks are dropped, and opened connections close once. The
storage driver finishes cleanup even if its caller stops awaiting it.

`WorkspaceStorageError` retains load/save intent, the saved directory for a
write, protocol stage, and original cause. Missing provider errors, incomplete
reads, and interrupted drivers have distinct structured classifications.
`WorkspaceError::Storage` exposes that error through its cause chain. Only an
undefined result means no saved record; null and other malformed values fail
handle admission. Restoration admits each query result into
`WorkspacePermission` before deciding whether to request read access. Granted
access restores the root; prompt or denied access requests permission and
admits that result again. `WorkspaceAccessRestriction` carries only prompt or
denied, so `PermissionNotGranted` cannot contain a granted state.

`WorkspaceError` retains record/root context and original provider causes.
Permission errors retain the directory, query/request
operation, and exact protocol stage. Property access, method invocation,
promise decoding, asynchronous rejection, and unknown statuses remain distinct
failures; malformed query results cannot trigger a subsequent request.

`ApplicationIdentity` owns the qualifier, organization, and application used by
the platform directory provider. Construction rejects separators and NUL and
requires a named application; qualifier and organization may be empty.
`directory` selects and creates its native data directory or obtains the
browser origin's private root. Browser identities share the origin's OPFS
authority. Native provider unavailability reports `ApplicationDirectoryError`
instead of selecting a process-relative data directory.

`EntryAccessError` distinguishes the attempted operation and retains the child
identity. Native failures retain the parent address and `std::io::Error`.
Browser failures retain the protocol stage and original provider value through
`BrowserIoCause`, including synchronous exceptions and promise rejections.
Decode failures retain the malformed value. Inspect the structured fields or
`std::error::Error::source`; format through `Display` at presentation.

Browser lookup, deletion, and content methods use the caught JavaScript protocol
boundary. Arguments are serialized there, and returned promises and handles are
checked.
Browser handles and causes must be accessed and dropped on their originating
thread; checked wrappers enforce that constraint.

`DirectoryEntry::list_entries` returns `DirectoryContents`, a complete listing
of immediate file and directory children in provider order. It never returns a
successful partial listing after a provider failure. Concurrent directory
changes retain the provider's enumeration semantics. Consume the listing
through its `IntoIterator` implementation; its backing collection stays private.

`DirectoryListingError` retains the directory and concrete provider cause.
Native failures distinguish opening the directory, advancing its iterator, and
inspecting a child. Symlinks retain file/directory target classification; other
native filesystem object kinds are omitted. A child metadata failure is
observable with its native address, including a dangling symlink.

Browser enumeration catches iterator acquisition, method/property exceptions,
promise rejection, and malformed results. It admits only known file/directory
kinds and checked provider handles. JavaScript completion values follow
`IteratorComplete` truthiness; an absent `done` field continues enumeration.
Completion skips the unused `value` field. Unknown kinds fail the listing.
The [ECMAScript iterator contract](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratorcomplete)
defines completion and property access behavior at that protocol boundary.

`FileContents` carries one exact serialized payload through file and resource
consumers. Empty files and every byte value are valid. `FileText` admits a whole
file as UTF-8 without changing spelling or newlines; `FileTextError` retains the
rejected bytes through its original decoding cause. Convert representations at
storage, serialization, or format-specific decoder construction.

File reads and writes return `FileContentError` with the file and concrete
cause.
Browser content errors retain the exact method and failure stage. The method
determines the operation, preventing a contradictory read/write classification.
Writes await both writing and closing; close failures remain observable.

`ResourceLocator` admits an exact address and owns routing, reads, writes, MIME
classification, and inline serialization. Construct it from external spelling;
empty, NUL-containing, and unsupported URI schemes are rejected. Supported
families are data, HTTP/HTTPS, blob, project/file addresses, and native paths.
Remote URL validation remains the request provider's responsibility.

`ResourceIoError` retains the address, operation, and structured failure. Inline
and remote resources reject writes. Project resources require the configured
project namespace; they never resolve against the process directory when that
authority is absent. Browser blob failures retain their original cause and
distinguish the first attempt from cache reuse.

Native direct-path spelling and file URI drive mapping remain unchanged.
Project paths require UTF-8 percent-decoding and admit each child before access;
native namespace matching uses path components and preserves the absolute root.
Native HTTP loads retain the provider's body handling; browser HTTP loads reject
unsuccessful statuses.

`FilePickerFilter::pick` selects one file with either `AnyFile` or an
`AcceptedType`. `FileTypeFilter` owns the description, `ResourceMime`, and a
nonempty `FileSuffixes` group. `FileSuffix` admits dotless or dotted spelling,
stores one leading dot, and preserves case, compound suffixes, ordering, and
repetition. Empty suffixes, trailing dots, invalid characters, and overlong
suffixes produce structured admission errors.

Browser suffix admission follows the
[File System Access suffix contract](https://wicg.github.io/file-system-access/#file-picker-options):
ASCII alphanumerics, `+`, and `.` are accepted, with at most 16 code points
including the leading dot. The provider matches MIME or suffix and retains its
default all-files option. These filters do not validate selected content.
Native RFD filters use dotless suffixes; that SDK does not accept MIME filters.

`pick_folder_entry` selects a directory. Both workflows return optional entry
authority. Native RFD exposes only an optional selection and supplies no cause
to distinguish cancellation from a failed dialog. The native `PickerError` is
therefore empty. Browser errors distinguish a missing window from caught
provider failures and retain picker kind, protocol stage, and `BrowserIoCause`.
Abort rejections remain errors because the provider can abort for reasons other
than cancellation. Malformed promises, arrays, cardinality, and handles fail
admission; an empty file array remains no selection. Single selection rejects
extra handles instead of silently discarding them.

Persistence and permission policy use nominal Rust owners. Raw provider values
are admitted only at the exact SDK, record, error-construction, or mandated
trait boundaries recorded in `rust-quality.toml`. No inline JavaScript storage
bridge remains. Other repository domains and embedded source coverage remain
part of the codebase-wide semantic type audit.
