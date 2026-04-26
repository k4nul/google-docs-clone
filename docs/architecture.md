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
- `src/storage`: snapshot store trait과 `src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록에 대응하는 adapter

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
- 현재는 `src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록에 대응하는 in-tree `SnapshotStore` adapter가 연결되며, future adapter는 같은 trait으로 다른 db/object storage를 대체할 수 있다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 snapshot store에서 문서를 복구한 뒤 처리할 수 있다.
- `GET /api/documents`는 active room과 snapshot store catalog를 합쳐 eviction 이후에도 문서 메타데이터를 유지한다.
- 기본 local ownership 모드에서는 앱 시작 시 snapshot catalog를 순회해 저장된 문서를 room registry로 eager hydrate한다.
- distributed ownership 모드(`ROOM_LOCATOR != local` 또는 authoritative `ROOM_COORDINATOR=file|sqlite|managed`)에서는 startup hydrate를 생략하고, 문서 catalog만 유지한 채 ownership 확인 뒤 `get_or_restore`에서 room을 on-demand로 복구한다.
- `FileSnapshotStore`는 catalog/hydrate 경로에서 corrupt snapshot 파일을 warning과 함께 건너뛰어 단일 손상 파일이 전체 startup/listing 실패로 번지지 않게 한다.
- `FileSnapshotStore`는 같은 디렉터리의 임시 파일에 snapshot을 먼저 쓴 뒤 `rename`으로 교체해 partial write가 마지막 정상 snapshot을 직접 덮어쓰지 않도록 한다.
- `AgdbSnapshotStore`는 `SNAPSHOT_AGDB_PATH` 단일 agdb 파일의 `snapshot:<doc_id>` alias node에 persisted snapshot JSON payload를 저장하고, startup hydrate/list 경로는 all-alias catalog scan 뒤 matching alias node를 다시 읽어 사용한다.
- `AmandineSnapshotStore`는 `SNAPSHOT_AMANDINE_PATH/snapshots.json` collection에 `doc_id -> persisted snapshot JSON` record를 저장하고, startup hydrate/list 경로는 collection 전체 parse를 사용한다.
- `ApexStoreSnapshotStore`는 `SNAPSHOT_APEX_STORE_PATH` 디렉터리의 ApexStore WAL/SSTable LSM engine에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고, startup hydrate/list 경로는 WAL replay 뒤 같은 catalog key를 그대로 사용한다.
- `ArmdbSnapshotStore`는 `SNAPSHOT_ARMDB_PATH` 디렉터리의 sharded ArmDB VarTree에 UUID bytes key와 persisted snapshot JSON bytes payload를 저장하고, save/delete 뒤 fsync-enabled flush를 호출하며 startup hydrate/list 경로는 tree iteration을 그대로 사용한다.
- `AssystemSnapshotStore`는 `SNAPSHOT_ASSYSTEM_PATH` 단일 assystem 파일에 `doc_id -> persisted snapshot JSON bytes` entry를 저장하고, startup hydrate/list 경로는 file-backed key list를 그대로 사용한다. upstream API가 I/O 오류를 panic으로 노출할 수 있어 adapter가 panic을 `StorageError`로 매핑하고 save/delete 뒤 파일 sync를 호출한다.
- `ColonDbSnapshotStore`는 `SNAPSHOT_COLON_DB_PATH` 단일 colon_db 파일에 `doc_id -> base64(persisted snapshot JSON)` row를 저장하고, startup hydrate/list 경로는 whole-file row scan을 그대로 사용한다. save/delete마다 파일 전체를 다시 쓰고 fsync하므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되며, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 제거해 temp artifact가 누적되지 않게 한다.
- `SqliteSnapshotStore`는 `SNAPSHOT_SQLITE_PATH` DB 파일의 `snapshots` 테이블에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 DB catalog를 그대로 사용한다.
- `HeedSnapshotStore`는 `SNAPSHOT_HEED_PATH` DB 디렉터리의 `snapshots` LMDB database에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 LMDB catalog를 그대로 사용한다.
- `HightowerKvSnapshotStore`는 `SNAPSHOT_HIGHTOWER_KV_PATH` 디렉터리의 `snapshot:<doc_id>` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 explicit flush를 수행하며 startup hydrate/list 경로는 same prefix scan을 그대로 사용한다.
- `HighlandcowsIsamSnapshotStore`는 `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH` path prefix의 `.idb`/`.idx` 파일 세트에 append-only data record와 on-disk B-tree index를 유지하고, 문서 catalog는 explicit `__catalog__` key로 고정해 startup hydrate/list 경로를 그대로 사용한다.
- `FjallSnapshotStore`는 `SNAPSHOT_FJALL_PATH` DB 디렉터리의 `snapshots` keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `PersistMode::SyncAll`로 journal을 동기화하며 startup hydrate/list 경로는 keyspace catalog를 그대로 사용한다.
- `PersySnapshotStore`는 `SNAPSHOT_PERSY_PATH` 단일 persy 파일의 `snapshots` segment와 `snapshots_by_doc_id` replace index에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 index catalog를 그대로 사용한다.
- `CandystoreSnapshotStore`는 `SNAPSHOT_CANDYSTORE_PATH` 디렉터리의 append-only engine keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, large payload는 `set_big/get_big` 경로를 사용하며 startup hydrate/list 경로는 explicit `__catalog__` key를 그대로 사용한다.
- `CelerixStoreSnapshotStore`는 `SNAPSHOT_CELERIX_STORE_PATH` 디렉터리의 Celerix Store persistence persona 파일 `snapshots.json` 안에서 `documents` app map을 사용해 `doc_id -> persisted snapshot JSON` value를 저장하고, save/delete마다 `save_persona` write-then-rename 경계로 startup hydrate/list 경로를 유지한다.
- `CuendillarSnapshotStore`는 `SNAPSHOT_CUENDILLAR_PATH` 루트 아래 `wal/`과 `sstable/` 디렉터리를 함께 관리하는 LSM engine에 문서 metadata와 Yrs full-state update를 저장하고, startup hydrate/list 경로는 `doc_id` keyspace full scan을 그대로 사용한다. 기본 dynamic config의 작은 WAL payload 한계를 올리고 WAL/version-manager sync policy를 `Always`로 고정해 restart recovery 경계를 보수적으로 잡는다.
- `NativeDbSnapshotStore`는 `SNAPSHOT_NATIVE_DB_PATH` 단일 native_db 파일의 primary-key catalog에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `NebariSnapshotStore`는 `SNAPSHOT_NEBARI_PATH` 디렉터리의 `snapshots` tree에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same tree range scan을 그대로 사용한다.
- `NikidbSnapshotStore`는 `SNAPSHOT_NIKIDB_PATH` 단일 nikidb 파일의 `snapshots` bucket과 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket catalog를 그대로 사용한다.
- `NodbSnapshotStore`는 `SNAPSHOT_NODB_PATH` 단일 nodb 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `ParityDbSnapshotStore`는 `SNAPSHOT_PARITY_DB_PATH` parity-db shim 디렉터리의 repository-local `store.json` column map에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same persisted catalog iteration을 그대로 사용한다.
- `PickleDbSnapshotStore`는 `SNAPSHOT_PICKLEDB_PATH` PickleDB 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `RcaskSnapshotStore`는 `SNAPSHOT_RCASK_PATH` RCask append-only segment 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 JSON string으로 함께 저장하고, 공개 delete API가 없어 tombstone string으로 삭제를 가리며 startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `MicroKvSnapshotStore`는 `SNAPSHOT_MICROKV_PATH` base path에 대응하는 MicroKV 파일 `<path>.kv`의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, auto-commit으로 저장을 확정하며 startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `RskeySnapshotStore`는 `SNAPSHOT_RSKEY_PATH` 단일 JSON hashmap 파일의 `doc_id -> persisted snapshot JSON string` 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete마다 전체 store를 다시 sync하며 startup hydrate/list 경로는 same hashmap key scan을 그대로 사용한다.
- `YakvSnapshotStore`는 `SNAPSHOT_YAKV_PATH` 단일 B-Tree 파일의 `snapshot:<doc_id>` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan catalog를 그대로 사용한다.
- `YakvdbSnapshotStore`는 `SNAPSHOT_YAKVDB_PATH` 단일 yakvdb B-Tree 파일의 `snapshot:<doc_id>` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 min/above key traversal catalog를 그대로 사용한다.
- `EpochDbSnapshotStore`는 `SNAPSHOT_EPOCH_DB_PATH` 디렉터리의 repository-local epoch-db shim `store.json` map에 문서 metadata와 Yrs full-state update를 JSON string으로 저장하고, explicit `__catalog__` key를 통해 startup hydrate/list 경로를 그대로 사용한다.
- `EtchdbSnapshotStore`는 `SNAPSHOT_ETCHDB_PATH` 디렉터리의 EtchDB WAL-backed store에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete를 `write_durable`로 확정해 startup WAL replay 뒤 같은 catalog key를 그대로 사용한다.
- `FastKvSnapshotStore`는 `SNAPSHOT_FASTKV_PATH` 단일 compressed binary dump 파일에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete마다 temp dump fsync 뒤 rename으로 startup reload 경계를 고정한다.
- `FerrumdbSnapshotStore`는 `SNAPSHOT_FERRUMDB_PATH` 단일 append-only log 파일에 `snapshot:<doc_id> -> persisted snapshot JSON` JSON value와 explicit `__catalog__` key를 함께 저장하고, `FsyncPolicy::Always`로 save/delete catalog 경계를 sync해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `RumDbSnapshotStore`는 `SNAPSHOT_RUMDB_PATH` 디렉터리의 append-only Bitcask-style log set에 문서 metadata와 Yrs full-state update를 JSON bytes로 저장하고, explicit `__catalog__` key와 startup log replay 뒤 keydir를 통해 hydrate/list 경로를 그대로 사용한다.
- `JsonStoreSnapshotStore`는 `SNAPSHOT_JSON_STORE_PATH` 단일 append-only JSON line 파일의 `doc_id -> persisted snapshot` 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 whole-file line replay와 key별 최신 offset 인덱스를 그대로 사용한다.
- `JsonDbRsSnapshotStore`는 `SNAPSHOT_JSON_DB_RS_PATH` 단일 JSON 배열 event log 파일에 save/delete record를 append하고, startup hydrate/list 경로는 whole-file event replay로 최신 문서 catalog와 snapshot을 복구한다.
- `Cdb64SnapshotStore`는 `SNAPSHOT_CDB64_PATH` 단일 CDB 파일에 `doc_id -> persisted snapshot JSON bytes` entry를 저장하고, save/delete마다 temp CDB rewrite와 atomic rename으로 restart 복구 경계를 고정한다. startup hydrate/list 경로는 whole-file key iteration을 그대로 사용한다.
- `JsonMutexDbSnapshotStore`는 `SNAPSHOT_JSON_MUTEX_DB_PATH` 단일 JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry를 저장하고, save/delete마다 `json-mutex-db` atomic save로 전체 파일을 교체하며 startup hydrate/list 경로는 whole-file object load를 사용한다.
- `ToiletdbSnapshotStore`는 `SNAPSHOT_TOILETDB_PATH` 단일 JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry를 저장하고, save/delete마다 ToiletDB temp file persist 뒤 file sync를 수행하며 startup hydrate/list 경로는 whole-file object load를 사용한다.
- `DirCacheSnapshotStore`는 `SNAPSHOT_DIR_CACHE_PATH` 디렉터리의 dir-cache entry set에 `snapshot-<doc_id>.json` payload와 explicit `__catalog__` key를 저장하고, save/delete 뒤 `sync()`를 호출해 startup hydrate/list 경로가 catalog key를 사용하도록 고정한다.
- `SqjsonSnapshotStore`는 `SNAPSHOT_SQJSON_PATH` 단일 sqjson DB 파일에 `snapshot:<doc_id>` payload를 base64 chunk key로 나눠 저장하고 `snapshot:<doc_id>:meta` key scan으로 startup hydrate/list 경로를 복구한다. save/delete는 새 version chunk를 먼저 쓴 뒤 metadata pointer를 교체하고 mmap flush와 파일/parent directory sync를 수행한다.
- `FeoxdbSnapshotStore`는 `SNAPSHOT_FEOXDB_PATH` 단일 FeOxDB 파일의 `snapshot:<doc_id>:<timestamp>:<event_id>` immutable event key에 문서 metadata와 Yrs full-state update를 함께 저장하고, delete는 tombstone event로 가리며 startup hydrate/list 경로는 prefix range scan 뒤 최신 event 선택을 사용한다.
- `LiteDbSnapshotStore`는 `SNAPSHOT_LITE_DB_PATH` 디렉터리의 append-only LiteDb keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `LogKvSnapshotStore`는 `SNAPSHOT_LOG_KV_PATH` append-only 단일 파일에 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 함께 저장하고, delete는 tombstone string으로 가리며 startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `AppendKvSnapshotStore`는 `SNAPSHOT_APPEND_KV_PATH` append-only 단일 파일에 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 함께 저장하고, delete는 append_kv tombstone record로 가리며 startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `AppendLogSnapshotStore`는 `SNAPSHOT_APPEND_LOG_PATH` append-only 단일 파일에 save/delete JSON event를 저장하고, startup hydrate/list 경로는 event log replay로 최신 snapshot map과 document catalog를 복구한다.
- `MhdbSnapshotStore`는 `SNAPSHOT_MHDB_PATH` path prefix가 만드는 `<path>.pag`/`<path>.dir` DBM 파일 쌍에 `snapshot:<doc_id>` payload와 explicit `__catalog__` blob을 chunked key로 나눠 저장하고, startup hydrate/list 경로는 같은 catalog blob을 그대로 사용한다.
- `LoroKvSnapshotStore`는 `SNAPSHOT_LORO_KV_PATH` 단일 binary SSTable 파일에 `doc_id -> persisted snapshot JSON bytes` payload를 저장하고, save/delete마다 `MemKvStore::export_all` 결과를 temp+rename으로 확정하며 startup hydrate/list 경로는 full scan을 그대로 사용한다.
- `LuckdbSnapshotStore`는 `SNAPSHOT_LUCKDB_PATH` 단일 LuckDB JSON document 파일에 `doc_id` field와 persisted snapshot JSON payload를 함께 저장하고, startup hydrate/list 경로는 `backend.snapshots` collection query를 그대로 사용한다.
- `DeebSnapshotStore`는 `SNAPSHOT_DEEB_PATH` 단일 Deeb JSON database 파일의 `snapshots` entity에 `doc_id` primary key와 persisted snapshot JSON payload를 함께 저장하고, save/delete마다 temp+rename commit으로 재시작 복구 경계를 고정한다.
- `RubinSnapshotStore`는 `SNAPSHOT_RUBIN_PATH` 단일 Rubin JSON 파일에 `doc_id -> persisted snapshot JSON` string entry를 저장하고, startup hydrate/list 경로는 Rubin `MemStore` string map scan을 그대로 사용한다.
- `LsmEngineSnapshotStore`는 `SNAPSHOT_LSM_ENGINE_PATH` WAL 파일에 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 WAL replay 뒤 같은 catalog key를 그대로 사용한다. upstream serde import 호환성은 vendored patch로 고정한다.
- `LsmStorageEngineSnapshotStore`는 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete 뒤 flush해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `LsmdbSnapshotStore`는 `SNAPSHOT_LSMDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 함께 저장하고, WAL sync-on-write 경계로 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `MindbSnapshotStore`는 `SNAPSHOT_MINDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있으면 upstream `RecoveryManager`로 WAL을 재생해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `MmdbSnapshotStore`는 `SNAPSHOT_MMDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 write batch로 함께 저장하고, sync write 뒤 flush해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `MuDbSnapshotStore`는 `SNAPSHOT_MU_DB_PATH` data 파일과 같은 디렉터리의 `index_<file_name>` index 파일에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 저장하고, save/delete 뒤 두 파일을 fsync해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `NanodbSnapshotStore`는 `SNAPSHOT_NANODB_PATH` 단일 JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry를 저장하고, save/delete 뒤 whole-file write로 startup hydrate/list 경로가 같은 root object를 그대로 사용하도록 고정한다.
- `SmolldbSnapshotStore`는 `SNAPSHOT_SMOLLDB_PATH` 단일 compressed SmollDB 파일의 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 temp+rename 경계로 함께 저장하고, startup hydrate/list 경로는 파일 load 뒤 같은 catalog key를 그대로 사용한다.
- `KstoneSnapshotStore`는 `SNAPSHOT_KSTONE_PATH` 디렉터리의 Kstone WAL/SSTable LSM keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete 뒤 flush해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `RoughdbSnapshotStore`는 `SNAPSHOT_ROUGHDB_PATH` 디렉터리의 RoughDB LevelDB-compatible WAL/SSTable keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 write batch로 함께 저장하고, sync write와 wait flush로 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `RaindbSnapshotStore`는 `SNAPSHOT_RAINDB_PATH` 디렉터리의 RainDB LevelDB-style WAL/SSTable keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 synchronous write batch로 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용하도록 고정한다.
- `InfusedbSnapshotStore`는 `SNAPSHOT_INFUSEDB_PATH` 단일 InfuseDB 파일의 `snapshots` collection에 base64 text로 인코딩된 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 collection catalog를 그대로 사용하도록 고정한다.
- `KafiSnapshotStore`는 `SNAPSHOT_KAFI_PATH` 단일 kafi bincode hashmap 파일의 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용하도록 고정한다.
- `TinkvSnapshotStore`는 `SNAPSHOT_TINKV_PATH` 디렉터리의 tinkv append-only data file set에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, sync write 경계로 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `LedgerKvSnapshotStore`는 `SNAPSHOT_LEDGER_KV_PATH` 디렉터리의 ledger-kv append-only journal에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, data/meta 파일 sync로 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다.
- `MaceSnapshotStore`는 `SNAPSHOT_MACE_PATH` 디렉터리의 Mace `snapshots` bucket에 `doc_id -> persisted snapshot JSON` 엔트리와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `JanqlSnapshotStore`는 `SNAPSHOT_JANQL_PATH` 디렉터리의 janql WAL/SSTable keyspace에 `doc_id -> persisted snapshot JSON` 엔트리와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `JasondbSnapshotStore`는 `SNAPSHOT_JASONDB_PATH` 단일 JasonDB append-only 파일에 `doc_id -> persisted snapshot JSON string` entry를 저장하고, startup hydrate/list 경로는 JasonDB index replay 결과를 사용한다.
- `JasonisnthappySnapshotStore`는 `SNAPSHOT_JASONISNTHAPPY_PATH` 단일 jasonisnthappy DB 파일의 `snapshots` collection에 `_id=<doc_id>` document로 persisted snapshot JSON payload를 저장하고, startup hydrate/list 경로는 collection scan 결과를 사용한다.
- `DataPileSnapshotStore`는 `SNAPSHOT_DATA_PILE_PATH` data-pile append-only record 디렉터리에 save/delete 이벤트를 JSON record로 저장하고, startup hydrate/list 경로는 record replay 결과를 사용한다.
- `DatastackSnapshotStore`는 `SNAPSHOT_DATASTACK_PATH` DataStack redb 파일의 `snapshots` collection에 `doc_id -> persisted snapshot JSON` document를 저장하고, startup hydrate/list 경로는 collection scan 결과를 사용한다.
- `JoydbSnapshotStore`는 `SNAPSHOT_JOYDB_PATH` 단일 Joydb JSON state 파일에 `JoydbSnapshotRecord`로 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `flush()`를 호출해 startup hydrate/list 경로를 같은 JSON state load에 고정한다.
- `PngDbSnapshotStore`는 `SNAPSHOT_PNG_DB_PATH` 단일 PNG 파일의 compressed text row chunks에 `doc_id`와 persisted snapshot JSON payload를 함께 저장하고, save/delete마다 전체 row set을 temp PNG로 쓴 뒤 rename해 startup hydrate/list 경로를 같은 PNG row scan에 고정한다.
- `ReadbSnapshotStore`는 `SNAPSHOT_READB_PATH` 디렉터리의 append-only data/index와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `RustliteSnapshotStore`는 `SNAPSHOT_RUSTLITE_PATH` 디렉터리의 WAL/SSTable engine과 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `RustcaskSnapshotStore`는 `SNAPSHOT_RUSTCASK_PATH` 디렉터리의 append-only Bitcask data/hint file과 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, sync mode를 켠 write 경계와 startup log replay 뒤 보조 catalog를 그대로 사용한다.
- `RustyLeveldbSnapshotStore`는 `SNAPSHOT_RUSTY_LEVELDB_PATH` 디렉터리의 LevelDB keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same keyspace full scan을 그대로 사용한다.
- `CanopydbSnapshotStore`는 `SNAPSHOT_CANOPYDB_PATH` 디렉터리의 `snapshots` tree와 transactional WAL/data file에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same tree iter scan을 그대로 사용한다.
- `CavesSnapshotStore`는 `SNAPSHOT_CAVES_PATH` 디렉터리의 `<doc_id>` key-per-file 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 같은 directory scan을 그대로 사용한다.
- `CkydbSnapshotStore`는 `SNAPSHOT_CKYDB_PATH` 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, delimiter-safe write를 위해 payload와 catalog를 base64 문자열로 저장하며 startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `CrepeDbSnapshotStore`는 `SNAPSHOT_CREPEDB_PATH` 단일 CrepeDB redb 파일의 basic `snapshots` table에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `CrystalSnapshotStore`는 `SNAPSHOT_CRYSTAL_PATH` 디렉터리의 `<doc_id>.bin` file에 persisted snapshot JSON string을 저장하고, startup hydrate/list 경로는 디렉터리 스캔 뒤 같은 key를 다시 읽어 사용한다.
- `ScdbSnapshotStore`는 `SNAPSHOT_SCDB_PATH` 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `SkvSnapshotStore`는 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data` / `<path>.index` 파일 쌍의 key-value 엔트리와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `SurrealkvSnapshotStore`는 `SNAPSHOT_SURREALKV_PATH` 단일 surrealkv B+tree 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan을 그대로 사용한다.
- `RustbreakSnapshotStore`는 `SNAPSHOT_RUSTBREAK_PATH` 단일 rustbreak path database catalog에 문서 metadata와 Yrs full-state update를 함께 저장하고, atomic file replace 기반 save 뒤 startup hydrate/list 경로는 same catalog scan을 그대로 사용한다.
- `YedbSnapshotStore`는 `SNAPSHOT_YEDB_PATH` yedb-compatible 디렉터리의 `snapshots/<doc_id>` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same namespace scan을 그대로 사용한다.
- `BtreeStoreSnapshotStore`는 `SNAPSHOT_BTREE_STORE_PATH` 단일 btree-store 파일의 `snapshots` bucket에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket scan을 그대로 사용한다.
- `SiamesedbSnapshotStore`는 `SNAPSHOT_SIAMESDB_PATH` 디렉터리의 `snapshots` map에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same map iteration을 그대로 사용한다.
- `StructsySnapshotStore`는 `SNAPSHOT_STRUCTSY_PATH` 단일 파일에 structsy persistent record로 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same record scan을 그대로 사용한다.
- `AeternusdbSnapshotStore`는 `SNAPSHOT_AETERNUSDB_PATH` 디렉터리의 WAL/SSTable LSM engine keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 보조 catalog를 그대로 사용한다.
- `ThunderdbSnapshotStore`는 `SNAPSHOT_THUNDERDB_PATH` 단일 파일의 `snapshots` bucket에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same bucket iter scan을 그대로 사용한다.
- `ThetadbSnapshotStore`는 `SNAPSHOT_THETADB_PATH` 단일 파일의 raw `doc_id` key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 cursor full scan을 그대로 사용한다.
- `TinybaseSnapshotStore`는 `SNAPSHOT_TINYBASE_PATH` sled 디렉터리의 typed `snapshots` table에 문서 metadata와 Yrs full-state update를 함께 저장하고, `doc_id` secondary index와 constant catalog index query로 startup hydrate/list 경로를 유지한다.
- `TinydbSnapshotStore`는 `SNAPSHOT_TINYDB_PATH` 단일 bincode dump 파일의 `doc_id` keyed record에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete마다 whole-file dump로 startup hydrate/list 경로를 유지한다.
- `DbliteSnapshotStore`는 `SNAPSHOT_DBLITE_PATH` 단일 파일의 string key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `DblessSnapshotStore`는 `SNAPSHOT_DBLESS_PATH` 단일 파일의 typed table 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `DbRsSnapshotStore`는 `SNAPSHOT_DB_RS_PATH` 디렉터리의 `LookupTable<String, PersistedSnapshot>` append-only typed table 로그에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 로그 replay 뒤 same table key scan을 그대로 사용한다.
- `DharmadbSnapshotStore`는 `SNAPSHOT_DHARMADB_PATH` 디렉터리의 dharmadb WAL/SSTable keyspace에 `doc_id -> persisted snapshot JSON` 엔트리와 explicit `__catalog__` key를 함께 저장한다. upstream DB 인스턴스가 비-Send라 adapter는 전용 worker thread에 DB를 고정하고, startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `DocDbSnapshotStore`는 `SNAPSHOT_DOCDB_PATH` 단일 JSON 파일의 key-value 엔트리에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `EmdbSnapshotStore`는 `SNAPSHOT_EMDB_PATH` 단일 emdb 파일에 `doc_id` key와 explicit `__catalog__` key를 저장하고, `EmdbBuilder::prefer_v4(true)`와 transaction + explicit `flush()` 경계로 v0.7 engine replay semantics를 고정한다.
- `OsmiumdbSnapshotStore`는 `SNAPSHOT_OSMIUMDB_PATH` 디렉터리의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete마다 `flush()` 뒤 `checkpoint()`를 호출해 WAL replay와 map snapshot reopen 경계를 함께 고정한다.
- `SanakirjaSnapshotStore`는 `SNAPSHOT_SANAKIRJA_PATH` 단일 파일의 copy-on-write B-tree keyspace에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same full scan을 그대로 사용한다.
- `SaturnSnapshotStore`는 `SNAPSHOT_SATURN_PATH` SaturnDB WAL 파일에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 함께 저장하고, startup hydrate/list 경로는 WAL replay 뒤 같은 catalog key를 그대로 사용한다.
- `FlashKvSnapshotStore`는 `SNAPSHOT_FLASH_KV_PATH` 디렉터리의 append-only keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `sync()`를 호출해 startup hydrate/list 경로가 같은 catalog를 그대로 사용하도록 고정한다.
- `GhaladbSnapshotStore`는 `SNAPSHOT_GHALADB_PATH` 디렉터리의 GhalaDB LSM key/value store에 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 함께 저장하고, save/delete 뒤 `sync()`를 호출해 startup hydrate/list 경로가 같은 catalog key를 그대로 사용하도록 고정한다. upstream bincode 2 API 호환성은 vendored patch로 고정한다.
- `SimpleDbSnapshotStore`는 `SNAPSHOT_SIMPLE_DB_PATH` 단일 line-oriented 파일에 `doc_id -> base64(persisted snapshot JSON)` 엔트리를 저장하고, startup hydrate/list 경로는 same key scan을 그대로 사용한다.
- `SnaildbSnapshotStore`는 `SNAPSHOT_SNAILDB_PATH` 디렉터리의 LSM keyspace와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, save/delete 뒤 `flush_memtable()`을 호출해 startup hydrate/list 경로가 같은 catalog를 그대로 사용하도록 고정한다.
- `KopperdbSnapshotStore`는 `SNAPSHOT_KOPPERDB_PATH` 디렉터리의 append-only 세그먼트 로그에 문서 metadata와 Yrs full-state update를 저장하고, 공개 delete API가 없어 `doc_id` key에 tombstone string을 덮어쓰면서 explicit `__catalog__` key를 함께 유지한다. startup hydrate/list 경로는 같은 catalog key와 append-only recovery를 그대로 사용한다.
- `IcefalldbSnapshotStore`는 `SNAPSHOT_ICEFALLDB_PATH` 디렉터리의 append-only `rsdb.log`에 문서 metadata와 Yrs full-state update를 저장하고, 공개 delete/iterator API가 없어 `doc_id` key tombstone과 explicit `__catalog__` key를 함께 유지한다. startup hydrate/list 경로는 같은 catalog key와 append-only log replay를 그대로 사용한다.
- `BitaskSnapshotStore`는 `SNAPSHOT_BITASK_PATH` 디렉터리의 append-only active/immutable log와 explicit `__catalog__` key에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 bitask log replay 뒤 재구축된 keydir를 그대로 사용한다.
- `BitkvRsSnapshotStore`는 `SNAPSHOT_BITKV_RS_PATH` 디렉터리의 append-only Bitcask-style log에 `doc_id -> persisted snapshot JSON` key-value를 sync write로 저장하고, startup hydrate/list 경로는 log replay 뒤 재구축된 in-memory index를 그대로 사용한다.
- `BitcaskEngineSnapshotStore`는 `SNAPSHOT_BITCASK_ENGINE_PATH` 디렉터리의 append-only bitcask-engine-rs log에 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key를 저장하고, startup hydrate/list 경로는 log replay 뒤 재구축된 in-memory index와 catalog key를 그대로 사용한다.
- `BlazeupSnapshotStore`는 `SNAPSHOT_BLAZEUP_PATH` 디렉터리 아래 blazeup/kv/sled `snapshots` bucket에 `snapshot:<doc_id> -> persisted snapshot JSON string` record와 explicit `__catalog__` key를 저장한다. blazeup path 설정은 process-global이라 adapter mutex로 init/operation을 직렬화해 startup hydrate/list 경로를 같은 catalog key에 고정한다.
- `CacacheSnapshotStore`는 `SNAPSHOT_CACACHE_PATH` content-addressed cache 디렉터리의 `snapshot:<doc_id>` key에 persisted snapshot JSON bytes를 저장하고, startup hydrate/list 경로는 cache index listing으로 복구한다.
- `CelerixStoreSnapshotStore`는 Celerix Store `Persistence`의 persona JSON save 경계를 사용하므로 backend 디렉터리 전체와 `snapshots.json`을 함께 백업해야 한다. catalog/list는 `documents` app map key 전체를 순회하고 corrupt entry는 warning과 함께 건너뛴다.
- `GrebedbSnapshotStore`는 `SNAPSHOT_GREBEDB_PATH` 디렉터리의 grebedb keyspace에 문서 metadata와 Yrs full-state update를 저장하고, explicit `__catalog__` key를 같은 `flush()` 경계로 함께 반영한다. startup hydrate/list 경로는 같은 catalog key를 그대로 사용한다.
- `GrumpydbSnapshotStore`는 `SNAPSHOT_GRUMPYDB_PATH` 디렉터리의 GrumpyDB page/B+Tree object store에 UUID key와 persisted snapshot JSON bytes payload를 저장하고, startup hydrate/list 경로는 full range scan을 그대로 사용한다.
- `GrausDbSnapshotStore`는 `SNAPSHOT_GRAUS_DB_PATH` 디렉터리의 GrausDb append-only log store에 `doc_id` key와 explicit `__catalog__` key를 저장하고, save/delete 뒤 DB handle 재오픈으로 startup replay 경계를 고정한다.
- `AbyssiniandbSnapshotStore`는 `SNAPSHOT_ABYSSINIANDB_PATH` 단일 파일의 `snapshots` map에 문서 metadata와 Yrs full-state update를 함께 저장하고, startup hydrate/list 경로는 same map key lookup과 보조 catalog scan을 그대로 사용한다.
- `S3SnapshotStore`는 `SNAPSHOT_S3_ENDPOINT` / `SNAPSHOT_S3_BUCKET` / `SNAPSHOT_S3_PREFIX` 조합 아래의 S3-compatible object storage에 `<prefix><doc_id>.json` object를 저장하고, startup hydrate/list 경로는 bucket listing 뒤 matching object를 다시 load해 catalog를 구성한다.
- `ManagedSnapshotStore`는 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external durability service `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`에 document metadata와 Yrs full-state update를 JSON으로 위임하고, startup hydrate/list 경로는 same service catalog를 사용한다.
- `IpjdbSnapshotStore`는 `SNAPSHOT_IPJDB_PATH` 디렉터리의 ipjdb `snapshots` collection에 `doc_id`와 persisted snapshot JSON payload를 item 파일로 저장하고, startup hydrate/list 경로는 collection full scan을 사용한다.
- `KagiSnapshotStore`는 `SNAPSHOT_KAGI_PATH` 단일 kagi bincode hashmap 파일에 `doc_id -> persisted snapshot JSON string` entry를 저장하고, startup hydrate/list 경로는 whole-file map load를 사용한다. upstream panic 기반 I/O는 adapter가 `StorageError::Io`로 매핑한다.
- `DeebSnapshotStore`는 `SNAPSHOT_DEEB_PATH` Deeb JSON database 파일의 `snapshots` entity에 `doc_id` primary key와 persisted snapshot JSON payload를 저장하고, save/delete 뒤 Deeb commit의 temp+rename 경계로 재시작 복구를 고정한다.
- `Config.snapshot_store`가 `src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록 전체에 대한 어댑터 선택을 담당한다. `apex_store` 모드에서는 `SNAPSHOT_APEX_STORE_PATH` 디렉터리의 ApexStore WAL/SSTable LSM engine과 explicit `__catalog__` key가 snapshot storage 단위가 되고, `armdb` 모드에서는 `SNAPSHOT_ARMDB_PATH` 디렉터리의 ArmDB VarTree가 snapshot storage 단위가 되고, `assystem` 모드에서는 `SNAPSHOT_ASSYSTEM_PATH` 단일 파일이 snapshot storage 단위가 되며, `blazeup` 모드에서는 `SNAPSHOT_BLAZEUP_PATH` 디렉터리의 blazeup `snapshots` bucket과 explicit `__catalog__` key가 snapshot storage 단위가 된다. `data_pile` 모드에서는 `SNAPSHOT_DATA_PILE_PATH` data-pile append-only record 디렉터리가 snapshot storage 단위가 되고, `emdb` 모드에서는 `SNAPSHOT_EMDB_PATH` 단일 emdb 파일의 `doc_id` key와 explicit `__catalog__` key가 snapshot storage 단위가 되며 adapter가 `EmdbBuilder::prefer_v4(true)`와 transaction + explicit `flush()` 경계로 v0.7 engine replay semantics를 고정한다. `osmiumdb` 모드에서는 `SNAPSHOT_OSMIUMDB_PATH` 디렉터리의 `snapshot:<doc_id>` key와 explicit `__catalog__` key가 snapshot storage 단위가 되며 adapter가 save/delete마다 `flush()` 뒤 `checkpoint()`를 호출해 WAL replay와 map snapshot reopen semantics를 함께 고정한다. `hurrahdb` 모드에서는 `SNAPSHOT_HURRAHDB_PATH` single append-only AOF 파일이 snapshot storage 단위가 된다. `fs_db` 모드에서는 `SNAPSHOT_FS_DB_PATH` 디렉터리의 `snapshot-<doc_id>.json` 파일들이 snapshot storage 단위가 된다. `mace` 모드에서는 `SNAPSHOT_MACE_PATH` 디렉터리의 Mace `snapshots` bucket과 explicit `__catalog__` key가 snapshot storage 단위가 된다. `jasondb` 모드에서는 `SNAPSHOT_JASONDB_PATH` 단일 append-only JasonDB 파일이 snapshot storage 단위가 되고, `jasonisnthappy` 모드에서는 `SNAPSHOT_JASONISNTHAPPY_PATH` 단일 jasonisnthappy DB 파일이 snapshot storage 단위가 되며, `joydb` 모드에서는 `SNAPSHOT_JOYDB_PATH` 단일 JSON state 파일이 snapshot storage 단위가 되며, `png_db` 모드에서는 `SNAPSHOT_PNG_DB_PATH` 단일 PNG 파일이 snapshot storage 단위가 되고, `cdb64` 모드에서는 `SNAPSHOT_CDB64_PATH` 단일 CDB 파일이 snapshot storage 단위가 되며, `luckdb` 모드에서는 `SNAPSHOT_LUCKDB_PATH` 단일 LuckDB JSON document 파일이 snapshot storage 단위가 된다. `kagi` 모드에서는 `SNAPSHOT_KAGI_PATH` 단일 bincode hashmap 파일이 snapshot storage 단위가 된다.
- `VsdbSnapshotStore`는 `SNAPSHOT_VSDB_PATH/store.meta.json`에 유지하는 `instance_id`를 통해 `VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb` 아래 `maps/<instance_id>.json` payload file을 다시 열고, 그 파일 안의 `doc_id -> persisted snapshot JSON bytes` map을 full scan해 startup hydrate/list 경로를 유지한다. adapter는 process-local global mutex로 접근을 직렬화하고 save/delete마다 file rewrite + sync를 호출해 restart 복구 경계를 단순하게 고정한다.

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
- 현재는 `src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록에 대응하는 snapshot adapter, shared SQLite lease 기반 owner coordination, 그리고 external managed lease coordination이 있으므로 ownership coordination plane과 snapshot durability plane을 모두 shared SQLite DB 밖으로 분리할 수 있다. `ManagedRoomCoordinator`/`ManagedRoomLocator`를 `SqliteSnapshotStore`와 결합한 multi-host handoff rehearsal, `ManagedSnapshotStore` 자체의 저장/복구 경계, `S3SnapshotStore` startup/config 복구 경계, 그리고 managed coordination과 managed snapshot durability를 함께 묶은 handoff rehearsal까지 모두 회귀 테스트로 검증됐다.

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
- 따라서 이 저장소에서 실제 handoff를 켜는 기본 경로는 검증이 끝난 shared snapshot durability `SNAPSHOT_STORE=sqlite`와, ownership plane 용도로 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite` 또는 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 조합하는 형태다. `SNAPSHOT_STORE=agdb`, `SNAPSHOT_STORE=apex_store`, `SNAPSHOT_STORE=armdb`, `SNAPSHOT_STORE=assystem`, `SNAPSHOT_STORE=blockbucket`, `SNAPSHOT_STORE=grebedb`, `SNAPSHOT_STORE=grumpydb`, `SNAPSHOT_STORE=graus_db`, `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=hmdb`, `SNAPSHOT_STORE=hurrahdb`, `SNAPSHOT_STORE=fs_db`, `SNAPSHOT_STORE=sqjson`, `SNAPSHOT_STORE=icefalldb`, `SNAPSHOT_STORE=bitask`, `SNAPSHOT_STORE=bitkv_rs`, `SNAPSHOT_STORE=bitcask_engine`, `SNAPSHOT_STORE=blazeup`, `SNAPSHOT_STORE=candystore`, `SNAPSHOT_STORE=celerix_store`, `SNAPSHOT_STORE=citadeldb`, `SNAPSHOT_STORE=data_pile`도 같은 `SnapshotStore` 경계에 연결됐다.
- `SNAPSHOT_STORE=cuendillar`, `SNAPSHOT_STORE=data_pile`, `SNAPSHOT_STORE=highlandcows_isam`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=mace`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=nebari`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=rcask`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=rskey`, `SNAPSHOT_STORE=readb`, `SNAPSHOT_STORE=rustlite`, `SNAPSHOT_STORE=rustcask`, `SNAPSHOT_STORE=rusty_leveldb`, `SNAPSHOT_STORE=canopydb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=crepedb`, `SNAPSHOT_STORE=crystal`, `SNAPSHOT_STORE=scdb`, `SNAPSHOT_STORE=skv`, `SNAPSHOT_STORE=surrealkv`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=thetadb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=db_rs`, `SNAPSHOT_STORE=dharmadb`, `SNAPSHOT_STORE=dir_cache`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=saturn`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=vsdb`, `SNAPSHOT_STORE=yakv`, `SNAPSHOT_STORE=yakvdb`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=smolldb`, `SNAPSHOT_STORE=kstone`, `SNAPSHOT_STORE=roughdb`, `SNAPSHOT_STORE=raindb`, `SNAPSHOT_STORE=infusedb`, `SNAPSHOT_STORE=kafi`, `SNAPSHOT_STORE=tinkv`, `SNAPSHOT_STORE=ledger_kv`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=kopperdb`, `SNAPSHOT_STORE=eight`, `SNAPSHOT_STORE=epoch_db`, `SNAPSHOT_STORE=fastkv`, `SNAPSHOT_STORE=ferrumdb`, `SNAPSHOT_STORE=rumdb`, `SNAPSHOT_STORE=koit`, `SNAPSHOT_STORE=lite_db`, `SNAPSHOT_STORE=lmdb_rs_core`, `SNAPSHOT_STORE=log_kv`, `SNAPSHOT_STORE=mhdb`, `SNAPSHOT_STORE=marble`, `SNAPSHOT_STORE=loro_kv`, `SNAPSHOT_STORE=luckdb`, `SNAPSHOT_STORE=deeb`, `SNAPSHOT_STORE=lsm_engine`, `SNAPSHOT_STORE=lsm_storage_engine`, `SNAPSHOT_STORE=lsmdb`, `SNAPSHOT_STORE=lsm_tree`, `SNAPSHOT_STORE=mindb`, `SNAPSHOT_STORE=mmdb`, `SNAPSHOT_STORE=mu_db`, `SNAPSHOT_STORE=nanodb`, `SNAPSHOT_STORE=jfs`, `SNAPSHOT_STORE=json_store`, `SNAPSHOT_STORE=json_db_rs`, `SNAPSHOT_STORE=json_mutex_db`, `SNAPSHOT_STORE=toiletdb`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=shorterdb`, `SNAPSHOT_STORE=s3`, `SNAPSHOT_STORE=managed`도 같은 `SnapshotStore` 경계에 연결됐고, managed coordination과 함께 묶은 실제 handoff rehearsal까지 회귀 테스트로 검증됐다.
- `CitadeldbSnapshotStore`는 encrypted CitadelDB 파일의 `snapshots` table에 persisted snapshot JSON bytes와 explicit catalog key를 저장한다. `SNAPSHOT_CITADELDB_PATH`, 같은 경로의 `.citadel-keys` sidecar, `SNAPSHOT_CITADELDB_PASSPHRASE`가 함께 복구 단위이며, shared coordination plane이 아니라 local durable restart recovery backend로 취급한다.
