# Architecture

## Module Structure

- `src/app.rs`: 앱 조립, CORS, HTTP tracing layer
- `src/config.rs`: 환경변수 로딩, 기본값, 값 검증
- `src/state.rs`: 전역 `AppState`, room registry 접근, 허용된 프런트엔드 origin 보관
- `src/errors.rs`: 공통 에러 타입과 HTTP 응답 변환
- `src/auth.rs`: Bearer 토큰 파싱과 인증 경계
- `src/routes`: REST endpoint 집합
- `src/collab`: Yrs room registry와 WebSocket 협업 경계
- `src/collab/coordinator.rs`: room ownership coordination lifecycle 확장 경계
- `src/models`: 문서 placeholder 모델
- `src/storage`: snapshot store trait과 memory/file/flash_kv/simple_db/docdb/shorterdb/sqlite/heed/hightower_kv/hmdb/bitask/candystore/jammdb/fjall/persy/persistent_kv/native_db/nikidb/nodb/parity_db/pickledb/microkv/redb/rskey/readb/rustlite/canopydb/caves/ckydb/scdb/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/tinybase/dblite/dbless/sanakirja/snaildb/tinykv/yakv/saberdb/jsondb/s3/managed adapter

## Request Flow

1. `main.rs`가 환경변수를 읽고 tracing을 초기화한다.
2. `app.rs`가 `AppState`와 라우트를 조합해 `Router`를 만든다.
3. `/api/*` 요청은 `routes` 모듈로 들어간다.
4. 문서 목록/생성은 관리용 `API_TOKEN`을 검증하고, 문서 상세/삭제는 문서별 `access_token`을 검증한다.
5. 문서 단위 room에 닿는 요청은 `AppState`의 `RoomLocator` 경계로 현재 노드 ownership을 먼저 확인한다.
6. route handler는 `AppState`를 통해 registry를 조회하고 JSON 응답을 반환한다.

## WebSocket / Collab Flow

- Incoming awareness updates pass through a custom Yrs protocol layer before shared room awareness state is mutated.

1. 클라이언트가 `GET /ws/:doc_id`로 업그레이드를 요청한다.
2. `collab/ws.rs`가 `doc_id` 형식을 검증하고 `Origin` 헤더가 `FRONTEND_ORIGIN`과 일치하는지 확인한다.
3. 같은 핸들러가 `Authorization: Bearer <access_token>`을 검증한다.
4. 같은 경계가 `RoomLocator`로 현재 노드 ownership을 확인한다.
5. 검증이 통과하면 `doc_id`에 해당하는 room을 조회하거나 snapshot store에서 on-demand로 복구한다.
6. room은 `Yrs Doc`, `Awareness`, lazy `BroadcastGroup`을 가진다.
7. 클라이언트는 연결 직후 awareness state를 `user`, optional `selection`, `client` 구조로 게시한다.
8. 업그레이드된 socket은 `AxumSink` / `AxumStream`으로 감싸진다.
9. `BroadcastGroup::subscribe`가 해당 문서의 협업 세션을 처리한다.
10. 마지막 WebSocket 세션이 종료되면 room snapshot을 저장하고 idle room을 registry에서 제거한다.

## Room Registry Structure

- 저장소는 `DashMap<Uuid, Arc<Room>>`
- key는 문서 ID
- value는 placeholder 문서 메타데이터와 Yrs awareness, lazy broadcast group
- 문서 메타데이터에는 외부 응답으로 노출하지 않는 `access_token`이 포함된다.
- 문서 API와 WebSocket 엔트리포인트가 같은 registry를 공유한다.
- awareness payload의 canonical shape는 `AwarenessState { user, selection?, client }`이며, 사용자 식별과 색상 규칙은 서버 모델과 문서에서 함께 관리한다.

## Persistence Extension Points

- `SnapshotStore` trait이 `load/save/delete` 경계를 정의하고, `RoomRegistry`가 이 trait에만 의존한다.
- `RoomLocator` trait이 "현재 프로세스가 이 문서의 authoritative owner인가"라는 진입 경계를 정의하고, `AppState`가 route/WS 진입 전에 이 trait만 호출한다.
- `RoomCoordinator` trait이 "이 문서 room이 현재 노드에서 active 상태로 전이/종료되는 시점"을 정의하고, WebSocket 첫 세션 시작 및 마지막 세션 종료 뒤에만 hook이 호출된다.
- `room_locator_from_config`는 현재 `LocalRoomLocator`, `StaticRoomLocator`, `FileRoomLocator`, `SqliteRoomLocator`, 또는 `ManagedRoomLocator`를 런타임에 선택한다.
- `room_coordinator_from_config`는 현재 `NoopRoomCoordinator`, `LoggingRoomCoordinator`, `FileRoomCoordinator`, `SqliteRoomCoordinator`, 또는 `ManagedRoomCoordinator`를 런타임에 선택한다.
- `LoggingRoomCoordinator`는 `NODE_ID`와 `doc_id` 기준 lifecycle log만 남기는 dry-run 구현이며, 외부 lease/heartbeat coordinator가 붙기 전 운영 관측용 경계로 사용한다.
- `FileRoomCoordinator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`에 canonical lease state를 atomic write로 남기고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 뒤 compare-and-release로 정리하는 file-backed 준비 구현이다. `NODE_BASE_URL`이 있으면 canonical origin 형태의 `base_url`도 lease record에 포함한다.
- `SqliteRoomCoordinator`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에 canonical lease state를 upsert하고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 뒤 `node_id + lease_id + epoch` compare-and-delete로 정리하는 authoritative SQLite 구현이다. `NODE_BASE_URL`이 있으면 canonical origin 형태의 `base_url`도 lease row에 포함한다.
- `ManagedRoomCoordinator`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에 `POST /v1/leases/:doc_id/acquire|renew|release`를 호출해 같은 canonical lease state를 유지하고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 뒤 compare-and-release를 요청하는 managed authority 구현이다. optional `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`이 있으면 모든 요청에 Bearer 토큰을 실어 보낸다.
- `Room::snapshot()`은 Yrs document를 full-state update로 직렬화하고 문서 metadata를 함께 저장한다.
- `Room::from_snapshot()`은 저장된 update를 다시 apply해 room을 복구한다.
- 각 room은 active WebSocket session 수를 추적하고, 마지막 세션 종료 시에만 snapshot 저장 후 eviction을 시도한다.
- 문서 삭제는 active WebSocket session 수가 0일 때만 허용하며, 세션이 남아 있으면 `409 conflict`로 거절한다.
- 현재는 `InMemorySnapshotStore`, `FileSnapshotStore`, `FlashKvSnapshotStore`, `SimpleDbSnapshotStore`, `DocDbSnapshotStore`, `ShorterDbSnapshotStore`, `SqliteSnapshotStore`, `HeedSnapshotStore`, `HightowerKvSnapshotStore`, `BitaskSnapshotStore`, `CandystoreSnapshotStore`, `JammdbSnapshotStore`, `FjallSnapshotStore`, `PersySnapshotStore`, `NativeDbSnapshotStore`, `NodbSnapshotStore`, `ParityDbSnapshotStore`, `PickleDbSnapshotStore`, `MicroKvSnapshotStore`, `RedbSnapshotStore`, `RskeySnapshotStore`, `ReadbSnapshotStore`, `RustliteSnapshotStore`, `CanopydbSnapshotStore`, `CavesSnapshotStore`, `CkydbSnapshotStore`, `ScdbSnapshotStore`, `SurrealkvSnapshotStore`, `SledSnapshotStore`, `RustbreakSnapshotStore`, `YedbSnapshotStore`, `BtreeStoreSnapshotStore`, `SiamesedbSnapshotStore`, `StructsySnapshotStore`, `AbyssiniandbSnapshotStore`, `AeternusdbSnapshotStore`, `ThunderdbSnapshotStore`, `TinybaseSnapshotStore`, `DbliteSnapshotStore`, `DblessSnapshotStore`, `SanakirjaSnapshotStore`, `SnaildbSnapshotStore`, `TinykvSnapshotStore`, `YakvSnapshotStore`, `S3SnapshotStore`, `ManagedSnapshotStore`가 연결되며, future adapter는 같은 trait으로 다른 db/object storage를 대체할 수 있다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 snapshot store에서 문서를 복구한 뒤 처리할 수 있다.
- `GET /api/documents`는 active room과 snapshot store catalog를 합쳐 eviction 이후에도 문서 메타데이터를 유지한다.
- 기본 local ownership 모드에서는 앱 시작 시 snapshot catalog를 순회해 저장된 문서를 room registry로 eager hydrate한다.
- distributed ownership 모드(`ROOM_LOCATOR != local` 또는 authoritative `ROOM_COORDINATOR=file|sqlite|managed`)에서는 startup hydrate를 생략하고, 문서 catalog만 유지한 채 ownership 확인 뒤 `get_or_restore`에서 room을 on-demand로 복구한다.
- `FileSnapshotStore`는 catalog/hydrate 경로에서 corrupt snapshot 파일을 warning과 함께 건너뛰어 단일 손상 파일이 전체 startup/listing 실패로 번지지 않게 한다.
- `FileSnapshotStore`는 같은 디렉터리의 임시 파일에 snapshot을 먼저 쓴 뒤 `rename`으로 교체해 partial write가 마지막 정상 snapshot을 직접 덮어쓰지 않도록 한다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되며, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 제거해 temp artifact가 누적되지 않게 한다.
- `SqliteSnapshotStore`는 `SNAPSHOT_SQLITE_PATH` DB 파일의 `snapshots` 테이블에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 DB catalog를 그대로 사용한다.
- `HeedSnapshotStore`는 `SNAPSHOT_HEED_PATH` DB 디렉터리의 `snapshots` LMDB database에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 LMDB catalog를 그대로 사용한다.
- `HightowerKvSnapshotStore`는 `SNAPSHOT_HIGHTOWER_KV_PATH` 디렉터리의 `snapshot:<doc_id>` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 explicit flush를 수행하며 startup hydrate/list 경로는 same prefix scan을 그대로 사용한다.
- `FjallSnapshotStore`는 `SNAPSHOT_FJALL_PATH` DB 디렉터리의 `snapshots` keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `PersistMode::SyncAll`로 journal을 동기화하며 startup hydrate/list 경로는 keyspace catalog를 그대로 사용한다.
- `PersySnapshotStore`는 `SNAPSHOT_PERSY_PATH` 단일 persy 파일의 `snapshots` segment와 `snapshots_by_doc_id` replace index에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 index catalog를 그대로 사용한다.
- `CandystoreSnapshotStore`는 `SNAPSHOT_CANDYSTORE_PATH` 디렉터리의 append-only engine keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, large payload는 `set_big/get_big` 경로를 사용하며 startup hydrate/list 경로는 explicit `__catalog__` key를 그대로 사용한다.
- `NativeDbSnapshotStore`는 `SNAPSHOT_NATIVE_DB_PATH` 단일 native_db 파일의 primary-key catalog에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `NikidbSnapshotStore`는 `SNAPSHOT_NIKIDB_PATH` 단일 nikidb 파일의 `snapshots` bucket과 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket catalog를 그대로 사용한다.
- `NodbSnapshotStore`는 `SNAPSHOT_NODB_PATH` 단일 nodb 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `ParityDbSnapshotStore`는 `SNAPSHOT_PARITY_DB_PATH` parity-db 디렉터리의 ordered `snapshots` column에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same BTree catalog iteration을 그대로 사용한다.
- `PickleDbSnapshotStore`는 `SNAPSHOT_PICKLEDB_PATH` PickleDB 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `MicroKvSnapshotStore`는 `SNAPSHOT_MICROKV_PATH` base path에 대응하는 MicroKV 파일 `<path>.kv`의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, auto-commit으로 저장을 확정하며 startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `RskeySnapshotStore`는 `SNAPSHOT_RSKEY_PATH` 단일 JSON hashmap 파일의 `doc_id -> persisted snapshot JSON string` 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete마다 전체 store를 다시 sync하며 startup hydrate/list 경로는 same hashmap key scan을 그대로 사용한다.
- `YakvSnapshotStore`는 `SNAPSHOT_YAKV_PATH` 단일 B-Tree 파일의 `snapshot:<doc_id>` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan catalog를 그대로 사용한다.
- `ReadbSnapshotStore`는 `SNAPSHOT_READB_PATH` 디렉터리의 append-only data/index와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `RustliteSnapshotStore`는 `SNAPSHOT_RUSTLITE_PATH` 디렉터리의 WAL/SSTable engine과 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `CanopydbSnapshotStore`는 `SNAPSHOT_CANOPYDB_PATH` 디렉터리의 `snapshots` tree와 transactional WAL/data file에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same tree iter scan을 그대로 사용한다.
- `CavesSnapshotStore`는 `SNAPSHOT_CAVES_PATH` 디렉터리의 `<doc_id>` key-per-file 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 같은 directory scan을 그대로 사용한다.
- `CkydbSnapshotStore`는 `SNAPSHOT_CKYDB_PATH` 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, delimiter-safe write를 위해 payload와 catalog를 base64 문자열로 저장하며 startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `ScdbSnapshotStore`는 `SNAPSHOT_SCDB_PATH` 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `SurrealkvSnapshotStore`는 `SNAPSHOT_SURREALKV_PATH` 단일 surrealkv B+tree 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan을 그대로 사용한다.
- `RustbreakSnapshotStore`는 `SNAPSHOT_RUSTBREAK_PATH` 단일 rustbreak path database catalog에 문서 metadata와 Yrs full-state update를 함께 저장하고, atomic file replace 기반 save 뒤 startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `YedbSnapshotStore`는 `SNAPSHOT_YEDB_PATH` 디렉터리의 `snapshots/<doc_id>` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same namespace scan을 그대로 사용한다.
- `BtreeStoreSnapshotStore`는 `SNAPSHOT_BTREE_STORE_PATH` 단일 btree-store 파일의 `snapshots` bucket에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket scan을 그대로 사용한다.
- `SiamesedbSnapshotStore`는 `SNAPSHOT_SIAMESDB_PATH` 디렉터리의 `snapshots` map에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same map iteration을 그대로 사용한다.
- `StructsySnapshotStore`는 `SNAPSHOT_STRUCTSY_PATH` 단일 파일에 structsy persistent record로 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same record scan을 그대로 사용한다.
- `AeternusdbSnapshotStore`는 `SNAPSHOT_AETERNUSDB_PATH` 디렉터리의 WAL/SSTable LSM engine keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `ThunderdbSnapshotStore`는 `SNAPSHOT_THUNDERDB_PATH` 단일 파일의 `snapshots` bucket에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket iter scan을 그대로 사용한다.
- `TinybaseSnapshotStore`는 `SNAPSHOT_TINYBASE_PATH` sled 디렉터리의 typed `snapshots` table에 문서 metadata와 Yrs full-state update를 함께 저장하고, `doc_id` secondary index와 constant catalog index query로 startup hydrate/list 경로를 유지한다.
- `DbliteSnapshotStore`는 `SNAPSHOT_DBLITE_PATH` 단일 파일의 string key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `DblessSnapshotStore`는 `SNAPSHOT_DBLESS_PATH` 단일 파일의 typed table 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `DocDbSnapshotStore`는 `SNAPSHOT_DOCDB_PATH` 단일 JSON 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `SanakirjaSnapshotStore`는 `SNAPSHOT_SANAKIRJA_PATH` 단일 파일의 copy-on-write B-tree keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan을 그대로 사용한다.
- `FlashKvSnapshotStore`는 `SNAPSHOT_FLASH_KV_PATH` 디렉터리의 append-only keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `sync()`를 호출해 startup hydrate/list 경로가 같은 catalog를 그대로 사용하도록 고정한다.
- `SimpleDbSnapshotStore`는 `SNAPSHOT_SIMPLE_DB_PATH` 단일 line-oriented 파일에 `doc_id -> base64(persisted snapshot JSON)` 엔트리를 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `SnaildbSnapshotStore`는 `SNAPSHOT_SNAILDB_PATH` 디렉터리의 LSM keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `flush_memtable()`을 호출해 startup hydrate/list 경로가 같은 catalog를 그대로 사용하도록 고정한다.
- `BitaskSnapshotStore`는 `SNAPSHOT_BITASK_PATH` 디렉터리의 append-only active/immutable log와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 bitask log replay 뒤 재구축된 keydir를 그대로 사용한다.
- `AbyssiniandbSnapshotStore`는 `SNAPSHOT_ABYSSINIANDB_PATH` 단일 파일의 `snapshots` map에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same map key lookup과 보조 catalog scan을 그대로 사용한다.
- `S3SnapshotStore`는 `SNAPSHOT_S3_ENDPOINT` / `SNAPSHOT_S3_BUCKET` / `SNAPSHOT_S3_PREFIX` 조합 아래의 S3-compatible object storage에 `<prefix><doc_id>.json` object를 저장하고, startup hydrate/list 경로는 bucket listing 뒤 matching object를 다시 load해 catalog를 구성한다.
- `ManagedSnapshotStore`는 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external durability service `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`에 document metadata와 Yrs full-state update를 JSON으로 위임하고, startup hydrate/list 경로는 same service catalog를 사용한다.
- `Config.snapshot_store`가 `memory`/`file`/`flash_kv`/`simple_db`/`docdb`/`shorterdb`/`sqlite`/`heed`/`hightower_kv`/`hmdb`/`bitask`/`candystore`/`jammdb`/`fjall`/`persy`/`persistent_kv`/`native_db`/`nikidb`/`nodb`/`parity_db`/`pickledb`/`microkv`/`redb`/`rskey`/`readb`/`rustlite`/`canopydb`/`caves`/`ckydb`/`scdb`/`surrealkv`/`sled`/`rustbreak`/`yedb`/`btree_store`/`siamesedb`/`structsy`/`abyssiniandb`/`aeternusdb`/`thunderdb`/`tinybase`/`dblite`/`dbless`/`sanakirja`/`snaildb`/`tinykv`/`saberdb`/`jsondb`/`s3`/`managed` 어댑터 선택을 담당하고, `file` 모드에서는 `SNAPSHOT_DIR/<doc_id>.json` 파일이, `flash_kv` 모드에서는 `SNAPSHOT_FLASH_KV_PATH` 디렉터리의 doc_id key와 `__catalog__` key가, `simple_db` 모드에서는 `SNAPSHOT_SIMPLE_DB_PATH` 단일 파일의 `doc_id` 라인이, `docdb` 모드에서는 `SNAPSHOT_DOCDB_PATH` 단일 JSON 파일의 doc_id key가, `shorterdb` 모드에서는 `SNAPSHOT_SHORTERDB_PATH` 디렉터리의 doc_id key와 `__catalog__` key가, `sqlite` 모드에서는 `SNAPSHOT_SQLITE_PATH` DB row가, `heed` 모드에서는 `SNAPSHOT_HEED_PATH` LMDB key가, `hightower_kv` 모드에서는 `SNAPSHOT_HIGHTOWER_KV_PATH` 디렉터리의 `snapshot:<doc_id>` key가, `hmdb` 모드에서는 `SNAPSHOT_HMDB_PATH` 디렉터리 아래 schema 로그 파일의 `doc_id` key가, `bitask` 모드에서는 `SNAPSHOT_BITASK_PATH` 디렉터리 아래 append-only log의 `doc_id` key와 `__catalog__` key가, `candystore` 모드에서는 `SNAPSHOT_CANDYSTORE_PATH` 디렉터리 아래 `doc_id` key와 `__catalog__` key가, `jammdb` 모드에서는 `SNAPSHOT_JAMMDB_PATH` bucket key가, `fjall` 모드에서는 `SNAPSHOT_FJALL_PATH` keyspace key가, `persy` 모드에서는 `SNAPSHOT_PERSY_PATH` index key가, `persistent_kv` 모드에서는 `SNAPSHOT_PERSISTENT_KV_PATH` 디렉터리의 key-value catalog가, `native_db` 모드에서는 `SNAPSHOT_NATIVE_DB_PATH` primary key가, `nikidb` 모드에서는 `SNAPSHOT_NIKIDB_PATH` bucket key와 `__catalog__` key가, `nodb` 모드에서는 `SNAPSHOT_NODB_PATH` key가, `parity_db` 모드에서는 `SNAPSHOT_PARITY_DB_PATH` ordered key가, `pickledb` 모드에서는 `SNAPSHOT_PICKLEDB_PATH` DB key가, `microkv` 모드에서는 `SNAPSHOT_MICROKV_PATH` base path가 생성할 `<path>.kv` DB keyspace가, `redb` 모드에서는 `SNAPSHOT_REDB_PATH` DB key가, `rskey` 모드에서는 `SNAPSHOT_RSKEY_PATH` JSON hashmap key가, `readb` 모드에서는 `SNAPSHOT_READB_PATH` 디렉터리의 key/value catalog가, `rustlite` 모드에서는 `SNAPSHOT_RUSTLITE_PATH` 디렉터리의 `snapshot:<doc_id>` key와 `__catalog__` key가, `canopydb` 모드에서는 `SNAPSHOT_CANOPYDB_PATH` 디렉터리의 `snapshots` tree key가, `caves` 모드에서는 `SNAPSHOT_CAVES_PATH` 디렉터리의 `<doc_id>` key-per-file entry가, `ckydb` 모드에서는 `SNAPSHOT_CKYDB_PATH` 디렉터리의 `doc_id` key와 `__catalog__` key가, `scdb` 모드에서는 `SNAPSHOT_SCDB_PATH` 디렉터리의 `doc_id` key와 `__catalog__` key가, `surrealkv` 모드에서는 `SNAPSHOT_SURREALKV_PATH` 단일 파일의 doc_id key가, `sled` 모드에서는 `SNAPSHOT_SLED_PATH` DB key가, `rustbreak` 모드에서는 `SNAPSHOT_RUSTBREAK_PATH` catalog key가, `yedb` 모드에서는 `SNAPSHOT_YEDB_PATH` 디렉터리 아래 `snapshots/<doc_id>` key가, `btree_store` 모드에서는 `SNAPSHOT_BTREE_STORE_PATH` 단일 파일의 `snapshots` bucket key가, `siamesedb` 모드에서는 `SNAPSHOT_SIAMESDB_PATH` 디렉터리의 `snapshots` map key가, `structsy` 모드에서는 `SNAPSHOT_STRUCTSY_PATH` 단일 파일의 structsy record key가, `abyssiniandb` 모드에서는 `SNAPSHOT_ABYSSINIANDB_PATH` 단일 파일의 `snapshots` map key가, `aeternusdb` 모드에서는 `SNAPSHOT_AETERNUSDB_PATH` 디렉터리의 `doc_id` key와 `__catalog__` key가, `thunderdb` 모드에서는 `SNAPSHOT_THUNDERDB_PATH` 단일 파일의 `snapshots` bucket key가, `tinybase` 모드에서는 `SNAPSHOT_TINYBASE_PATH` sled 디렉터리의 typed `snapshots` table record와 `doc_id`/catalog secondary index가, `dblite` 모드에서는 `SNAPSHOT_DBLITE_PATH` 단일 파일의 doc_id key가, `dbless` 모드에서는 `SNAPSHOT_DBLESS_PATH` 단일 파일의 doc_id key가, `sanakirja` 모드에서는 `SNAPSHOT_SANAKIRJA_PATH` 단일 파일의 doc_id key가, `snaildb` 모드에서는 `SNAPSHOT_SNAILDB_PATH` 디렉터리의 doc_id key와 `__catalog__` key가, `tinykv` 모드에서는 `SNAPSHOT_TINYKV_PATH` 단일 JSON 파일의 doc_id key가, `saberdb` 모드에서는 `SNAPSHOT_SABERDB_PATH` 단일 pretty JSON 파일의 doc_id key가, `jsondb` 모드에서는 `SNAPSHOT_JSONDB_PATH` 단일 versioned pretty JSON 파일의 `snapshots.<doc_id>` key가, `s3` 모드에서는 `SNAPSHOT_S3_PREFIX<doc_id>.json` object key가, `managed` 모드에서는 `SNAPSHOT_MANAGED_BASE_URL/v1/snapshots/:doc_id` resource가 snapshot storage 단위가 된다.

## Multi-Process Distribution Strategy

- 현재 `RoomRegistry`는 프로세스 로컬 `DashMap`에 의존하므로, 같은 `doc_id`를 여러 프로세스가 동시에 소유하면 Yrs update ordering과 awareness fan-out이 분리된다.
- 따라서 다중 프로세스 운영의 첫 번째 불변조건은 "한 시점에 한 문서는 정확히 한 협업 프로세스만 authoritative owner가 된다"로 둔다.
- 권장 1차 전략은 L7 sticky routing보다 명시적 room ownership 조회를 우선하는 것이다. sticky session만으로는 프로세스 재시작, scale-in, reconnect 시 ownership drift를 막기 어렵다.
- 진입 플로우는 `GET /ws/:doc_id` 전에 API gateway 또는 thin coordination layer가 `doc_id -> owner node` 매핑을 조회하고, 현재 노드가 owner가 아니면 redirect 또는 proxy 방식으로 owner에 라우팅하는 형태를 기준으로 한다.
- owner node는 room 활성 중에는 주기적으로 lease/heartbeat를 갱신하고, lease 만료 시에만 다른 노드가 snapshot restore 후 ownership을 인계받는다.
- snapshot store는 프로세스 공용 저장소여야 하며, owner handoff 직전 마지막 full-state update가 내구적으로 저장되어야 한다.
- awareness는 durability 대상이 아니므로 owner handoff 시 재게시를 허용하고, 문서 본문 CRDT update와 분리해 취급한다.
- cross-node fan-out이 필요해지는 시점 전까지는 한 room의 WebSocket 세션을 모두 owner node에 붙이는 방식이 가장 단순하다. node 간 pub/sub 복제는 ownership 우회가 아니라 장애 복구 보조 경로로만 고려한다.
- 구현 확장 포인트는 `RoomRegistry` 앞단에 `RoomLocator` 또는 동등한 ownership resolver를 두고, 현재 `get_or_restore` 호출 전에 authoritative node 결정을 끼워 넣는 형태가 가장 경계에 맞다.
- lease/heartbeat 기반 coordination store는 별도 `RoomCoordinator` 구현으로 붙여 첫 세션 시작 시 activate, 마지막 세션 종료 후 snapshot persist 성공 시 deactivate를 담당하게 두는 것이 현재 경계에 맞다.
- 현재 저장소에는 이 경계를 구현한 기본 `LocalRoomLocator`, 문서별 owner hints를 읽는 `StaticRoomLocator`, `FileRoomCoordinator` state를 읽는 `FileRoomLocator`, SQLite lease row를 읽는 `SqliteRoomLocator`, 그리고 external lease service를 읽는 `ManagedRoomLocator`가 들어가 있다.
- 현재 저장소에는 side effect 없는 `NoopRoomCoordinator`, dry-run `LoggingRoomCoordinator`, local/shared filesystem에 lease state와 heartbeat를 남기는 `FileRoomCoordinator`, shared SQLite DB에 lease state를 남기는 `SqliteRoomCoordinator`, 그리고 external lease service에 같은 lifecycle을 위임하는 `ManagedRoomCoordinator`가 들어가 있다.
- `StaticRoomLocator`는 문서별 owner 힌트를 읽어 현재 `NODE_ID`와 다른 owner를 가진 room 요청을 조기에 차단하고 optional `base_url` 힌트를 응답에 실어준다.
- `FileRoomLocator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active owner lease state를 읽어 현재 `NODE_ID`와 다른 node가 기록돼 있고 `expires_at`이 아직 지나지 않았으면 non-local owner로 간주한다. lease record에 `base_url`이 있으면 conflict 응답에도 함께 전달해 redirect/proxy 결정을 돕는다.
- `SqliteRoomLocator`는 `ROOM_COORDINATOR_SQLITE_PATH`의 active owner lease row를 읽어 현재 `NODE_ID`와 다른 node가 기록돼 있고 `expires_at`이 아직 지나지 않았으면 non-local owner로 간주한다. lease row에 `base_url`이 있으면 conflict 응답에도 함께 전달해 redirect/proxy 결정을 돕는다.
- `ManagedRoomLocator`는 `ROOM_COORDINATION_MANAGED_BASE_URL`의 `GET /v1/leases/:doc_id` 응답을 읽어 현재 `NODE_ID`와 다른 node가 기록돼 있고 `expires_at`이 아직 지나지 않았으면 non-local owner로 간주한다. lease record에 `base_url`이 있으면 conflict 응답에도 함께 전달해 redirect/proxy 결정을 돕는다.
- 현재는 `InMemorySnapshotStore`, 로컬 `FileSnapshotStore`, 단일 DB 파일 기반 `SqliteSnapshotStore`, vendor-specific embedded DB 기반 `HeedSnapshotStore`/`BitaskSnapshotStore`/`CandystoreSnapshotStore`/`JammdbSnapshotStore`/`FjallSnapshotStore`/`PersySnapshotStore`/`NativeDbSnapshotStore`/`ParityDbSnapshotStore`/`PickleDbSnapshotStore`/`MicroKvSnapshotStore`/`RedbSnapshotStore`/`ReadbSnapshotStore`/`RustliteSnapshotStore`/`CanopydbSnapshotStore`/`CkydbSnapshotStore`/`SurrealkvSnapshotStore`/`SledSnapshotStore`/`RustbreakSnapshotStore`/`YedbSnapshotStore`/`BtreeStoreSnapshotStore`/`SiamesedbSnapshotStore`/`StructsySnapshotStore`/`AbyssiniandbSnapshotStore`/`ThunderdbSnapshotStore`/`SnaildbSnapshotStore`/`SimpleDbSnapshotStore`/`DocDbSnapshotStore`, S3-compatible `S3SnapshotStore`, external `ManagedSnapshotStore`, shared SQLite lease 기반 owner coordination, 그리고 external managed lease coordination이 있으므로 ownership coordination plane과 snapshot durability plane을 모두 shared SQLite DB 밖으로 분리할 수 있다. `ManagedRoomCoordinator`/`ManagedRoomLocator`를 `SqliteSnapshotStore`와 결합한 multi-host handoff rehearsal, `ManagedSnapshotStore` 자체의 저장/복구 경계, `S3SnapshotStore` startup/config 복구 경계, 그리고 managed coordination과 managed snapshot durability를 함께 묶은 handoff rehearsal까지 모두 회귀 테스트로 검증됐다.

## Authoritative Coordination Store Contract

- authoritative backend는 `RoomCoordinator`가 쓰는 write path와 `RoomLocator`가 읽는 lookup path를 동일한 lease record로 맞춰야 한다. 현재 SQLite 구현과 managed HTTP backend도 같은 contract를 그대로 따른다.
- canonical lease record는 최소 아래 필드를 포함한다.

```json
{
  "doc_id": "00000000-0000-0000-0000-000000000000",
  "node_id": "node-a",
  "base_url": "https://collab-a.internal",
  "lease_id": "2b0fd35e-7f83-4558-a271-695bcdb22fd4",
  "epoch": 17,
  "acquired_at": "2026-04-20T09:00:00Z",
  "renewed_at": "2026-04-20T09:00:10Z",
  "expires_at": "2026-04-20T09:00:30Z"
}
```

- `lease_id`는 compare-and-swap 키다. `renew`/`release`는 현재 holder의 `lease_id`와 `node_id`가 일치할 때만 성공해야 한다.
- `epoch`는 fencing token이다. snapshot write나 future redirect metadata가 늦게 도착하더라도, 더 작은 `epoch`를 가진 stale owner의 side effect는 거절해야 한다.
- `expires_at`은 stale 판단의 유일한 authoritative 기준이다. file mtime, process uptime, local heuristic만으로 ownership을 조기 회수하지 않는다.
- `base_url`은 optional이며, 노출하는 경우 현재 HTTP `409 conflict` 응답의 `owner.base_url` 계약과 같은 canonical origin 규칙을 따라야 한다.

## Lease Lifecycle Policy

- acquire: 첫 세션 진입 직전 현재 lease가 없거나 `expires_at <= now`일 때만 새 lease를 쓴다.
- activate: acquire 성공 뒤에만 local room activation을 진행한다.
- renew: room이 active인 동안 background heartbeat loop가 `renewed_at`과 `expires_at`을 갱신한다.
- release: 마지막 세션 종료 후 snapshot persist 성공 뒤에만 compare-and-delete로 lease를 지운다.
- handoff: 이전 owner의 `expires_at`이 지난 뒤 새 owner가 acquire하고 snapshot restore를 수행한 다음 WebSocket 세션을 받는다.
- failure handling: snapshot persist 실패 시 즉시 release하지 않는다. TTL이 남아 있는 동안 기존 owner를 유지해 split-brain과 stale restore를 피한다.

## Recommended Timing Defaults

- `heartbeat_interval`: 10초
- `lease_ttl`: 30초
- `max_missed_heartbeats_before_stale`: 2
- renew scheduling은 TTL의 절반보다 짧아야 한다.
- takeover는 마지막 `expires_at` 경과 뒤에만 허용한다.

## Current Repository Boundary

- 현재 코드베이스의 `FileRoomCoordinator`/`FileRoomLocator`는 위 계약의 filesystem rehearsal 구현을 제공한다.
- 현 file 구현은 `lease_id`, `epoch`, optional `base_url`, `renewed_at`, `expires_at`를 기록하고 background heartbeat로 lease를 연장하지만, CAS 보장 범위가 shared filesystem과 단일 파일 교체에 한정된다.
- 현재 코드베이스의 `SqliteRoomCoordinator`/`SqliteRoomLocator`는 같은 계약을 shared SQLite DB row에 매핑해 transactional CAS를 제공한다.
- 따라서 이 저장소에서 실제 handoff를 켜는 기본 경로는 검증이 끝난 shared snapshot durability `SNAPSHOT_STORE=sqlite`와, ownership plane 용도로 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite` 또는 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 조합하는 형태다. `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=hmdb`, `SNAPSHOT_STORE=bitask`, `SNAPSHOT_STORE=candystore`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=rskey`, `SNAPSHOT_STORE=readb`, `SNAPSHOT_STORE=rustlite`, `SNAPSHOT_STORE=canopydb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=scdb`, `SNAPSHOT_STORE=surrealkv`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=yakv`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=shorterdb`, `SNAPSHOT_STORE=s3`, `SNAPSHOT_STORE=managed`도 같은 `SnapshotStore` 경계에 연결됐고, managed coordination과 함께 묶은 실제 handoff rehearsal까지 회귀 테스트로 검증됐다.
