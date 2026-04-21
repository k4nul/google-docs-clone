# Backend Collaborative Server

Axum, Tokio, Yrs 기반으로 시작하는 협업 편집 백엔드 부트스트랩 프로젝트입니다.

## 프로젝트 개요

문서 단위의 실시간 협업 서버를 Rust로 안전하게 시작할 수 있도록 최소 실행 구조를 제공합니다. 현재 단계에서는 HTTP 헬스체크, 문서 생성/조회/삭제 API, 문서별 WebSocket 진입점, 관리용 API 토큰과 문서별 access token 기반 접근 제어, in-memory room registry, 그리고 memory/file/agdb/flash_kv/blockbucket/grebedb/grumpydb/graus_db/highlandcows_isam/simple_db/docdb/eight/epoch_db/ferrumdb/rumdb/shorterdb/sqlite/heed/hightower_kv/hmdb/bitask/bitkv_rs/candystore/cuendillar/jammdb/mace/janql/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/jsondb/kv/koit/lite_db/log_kv/lsm_storage_engine/mindb/mmdb/nanodb/jfs/json_store/feoxdb/s3/managed snapshot 저장 추상화를 포함합니다.

## 해결하려는 문제

협업 편집 시스템은 HTTP API, WebSocket 세션, 문서별 상태 관리, CRDT 동기화 경계를 초기에 잘 나누지 않으면 빠르게 복잡해집니다. 이 레포는 그 복잡도를 초기에 제어하기 위해 compile-safe한 기본 골격과 문서화를 함께 제공합니다.

## 핵심 기능

- `GET /api/health` 헬스체크
- `GET /api/documents` active room과 persisted snapshot을 합친 문서 목록 조회
- `POST /api/documents` 문서 생성 및 room 초기화
- `GET /api/documents/:id` 기존 문서 상세 조회
- `DELETE /api/documents/:id` 문서 및 room 제거
- `GET /ws/:doc_id` 문서별 협업 WebSocket 진입점
- 관리용 API 토큰과 문서별 access token 기반 인증/접근 제어
- `DashMap` 기반 room registry와 idle room eviction
- `yrs-axum` 기반 broadcast group 연결
- `SnapshotStore` trait 및 memory/file/agdb/flash_kv/blockbucket/grebedb/grumpydb/graus_db/highlandcows_isam/simple_db/docdb/eight/epoch_db/ferrumdb/rumdb/shorterdb/sqlite/heed/hightower_kv/hmdb/bitask/bitkv_rs/candystore/cuendillar/jammdb/mace/janql/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/jsondb/kv/koit/lite_db/log_kv/lsm_storage_engine/mindb/mmdb/nanodb/jfs/json_store/feoxdb/s3/managed adapter
- `RoomLocator` 경계와 config-driven `local`/`static`/`file`/`sqlite`/`managed` ownership resolver
- `RoomCoordinator` 경계와 config-driven `noop`/`logging`/`file`/`sqlite`/`managed` session lifecycle hook

## 기술 스택

- Rust
- Axum
- Tokio
- Yrs
- yrs-axum
- DashMap
- Tracing / tracing-subscriber

## 로컬 실행 방법

```bash
cp .env.example .env
cargo run
```

기본 실행 주소는 `127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`으로 설정되어 있어 로컬 프런트엔드 개발 서버와 포트가 겹치지 않습니다.

## 검증 흐름

```bash
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
```

- `./scripts/preflight.sh commit`는 `.git` 메타데이터 쓰기 가능 여부를 먼저 확인해 commit/stage 차단을 조기에 드러낸다.
- `./scripts/preflight.sh publish`는 여기에 `github.com` DNS 확인을 더해 push 가능성을 사전에 확인한다.
- `./scripts/preflight.sh websocket`는 socket bind가 필요한 WebSocket 검증 레인이 현재 러너에서 실행 가능한지 probe test로 확인한다.
- `./scripts/verify.sh core`는 `cargo fmt --check`, `cargo check --locked`, 그리고 socket bind가 필요 없는 테스트만 실행한다. commit/push 가능 여부와는 분리돼 있어 sandbox 환경에서도 core 검증을 막지 않는다.
- `./scripts/verify.sh websocket`는 socket bind가 필요한 WebSocket/삭제 통합 테스트만 분리 실행한다.
- socket-required 테스트를 새로 추가하면 `scripts/verify.sh`의 core skip 목록과 websocket lane을 함께 갱신한다.

## API/WS 개요

- HTTP base path: `/api`
- Health: `GET /api/health`
- Documents: `GET /api/documents`, `POST /api/documents`, `GET /api/documents/:id`, `DELETE /api/documents/:id`
- Collaboration WebSocket: `GET /ws/:doc_id`

`GET /api/documents`와 `POST /api/documents`는 `Authorization: Bearer <API_TOKEN>` 헤더가 필요합니다. `POST /api/documents` 응답에는 해당 문서 전용 `access_token`이 포함되며, 이후 `GET /api/documents/:id`, `DELETE /api/documents/:id`, `GET /ws/:doc_id`는 모두 `Authorization: Bearer <access_token>` 헤더가 필요합니다. 존재하지 않는 문서 ID로 상세 조회나 WebSocket 연결을 시도하면 `404`를 반환합니다. 활성 협업 WebSocket 세션이 남아 있는 문서를 삭제하려 하면 `409 conflict`를 반환합니다. WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 합니다.

non-local owner 때문에 `409 conflict`가 반환될 때는 기존 JSON body와 함께 ingress/proxy 레이어가 바로 사용할 수 있도록 `x-collab-owner-node-id` 헤더가 추가됩니다. `owner.base_url`이 있으면 canonical owner origin을 담은 `x-collab-owner-base-url`, 현재 요청 path/query를 owner origin에 붙인 `x-collab-redirect-location`, 그리고 표준 `Location` 헤더도 함께 실립니다.

## 폴더 구조 요약

```text
.
|-- AGENTS.md
|-- README.md
|-- .env.example
|-- docs/
|-- scripts/
|-- src/
|   |-- app.rs
|   |-- collab/
|   |-- config.rs
|   |-- errors.rs
|   |-- models/
|   |-- routes/
|   |-- state.rs
|   |-- lib.rs
|   `-- main.rs
`-- tests/
```

## 환경변수 요약

- `HOST`: 서버 바인드 호스트
- `PORT`: 서버 바인드 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing 필터 설정
- `API_TOKEN`: 문서 생성/목록 조회용 관리 토큰
- `SNAPSHOT_STORE`: `memory`, `file`, `agdb`, `flash_kv`, `blockbucket`, `grebedb`, `grumpydb`, `graus_db`, `highlandcows_isam`, `simple_db`, `docdb`, `eight`, `epoch_db`, `ferrumdb`, `rumdb`, `shorterdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `icefalldb`, `bitask`, `bitkv_rs`, `candystore`, `cuendillar`, `jammdb`, `mace`, `janql`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nikidb`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `rcask`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `sanakirja`, `snaildb`, `tinykv`, `yakv`, `saberdb`, `smolldb`, `kstone`, `jsondb`, `kopperdb`, `kv`, `koit`, `lite_db`, `log_kv`, `lsm_storage_engine`, `mmdb`, `nanodb`, `jfs`, `json_store`, `feoxdb`, `s3`, 또는 `managed`
- `SNAPSHOT_STORE=ferrumdb`: FerrumDB append-only log 파일 store도 지원한다.
- `SNAPSHOT_STORE=mindb`: Mindb WAL/SSTable LSM 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=mmdb`: MMDB WAL/SSTable LSM 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=nanodb`: NanoDB single-file JSON store도 지원한다.
- `SNAPSHOT_DIR`: `SNAPSHOT_STORE=file`일 때 snapshot JSON 파일을 저장할 디렉터리
- `SNAPSHOT_AGDB_PATH`: `SNAPSHOT_STORE=agdb`일 때 snapshot agdb 단일 파일 경로
- `SNAPSHOT_FLASH_KV_PATH`: `SNAPSHOT_STORE=flash_kv`일 때 snapshot flash-kv 디렉터리 경로
- `SNAPSHOT_BLOCKBUCKET_PATH`: `SNAPSHOT_STORE=blockbucket`일 때 snapshot blockbucket 단일 파일 경로
- `SNAPSHOT_GREBEDB_PATH`: `SNAPSHOT_STORE=grebedb`일 때 snapshot grebedb 디렉터리 경로
- `SNAPSHOT_GRUMPYDB_PATH`: `SNAPSHOT_STORE=grumpydb`일 때 snapshot grumpydb 디렉터리 경로
- `SNAPSHOT_GRAUS_DB_PATH`: `SNAPSHOT_STORE=graus_db`일 때 snapshot GrausDb 로그 디렉터리 경로
- `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`: `SNAPSHOT_STORE=highlandcows_isam`일 때 snapshot highlandcows-isam path prefix. 실제 저장 파일은 `<path>.idb`, `<path>.idx`
- `SNAPSHOT_SIMPLE_DB_PATH`: `SNAPSHOT_STORE=simple_db`일 때 snapshot simple_db 단일 파일 경로
- `SNAPSHOT_DOCDB_PATH`: `SNAPSHOT_STORE=docdb`일 때 snapshot docdb JSON 파일 경로
- `SNAPSHOT_EIGHT_PATH`: `SNAPSHOT_STORE=eight`일 때 snapshot eight 디렉터리 경로
- `SNAPSHOT_EPOCH_DB_PATH`: `SNAPSHOT_STORE=epoch_db`일 때 snapshot epoch-db 디렉터리 경로
- `SNAPSHOT_FERRUMDB_PATH`: `SNAPSHOT_STORE=ferrumdb`일 때 snapshot FerrumDB append-only log 파일 경로
- `SNAPSHOT_RUMDB_PATH`: `SNAPSHOT_STORE=rumdb`일 때 snapshot rumdb append-only log 디렉터리 경로
- `SNAPSHOT_SHORTERDB_PATH`: `SNAPSHOT_STORE=shorterdb`일 때 snapshot shorterdb 디렉터리 경로
- `SNAPSHOT_SQLITE_PATH`: `SNAPSHOT_STORE=sqlite`일 때 snapshot SQLite DB 파일 경로
- `SNAPSHOT_HEED_PATH`: `SNAPSHOT_STORE=heed`일 때 snapshot heed DB 디렉터리 경로
- `SNAPSHOT_HIGHTOWER_KV_PATH`: `SNAPSHOT_STORE=hightower_kv`일 때 snapshot hightower-kv 데이터 디렉터리 경로
- `SNAPSHOT_HMDB_PATH`: `SNAPSHOT_STORE=hmdb`일 때 snapshot hmdb append-only 로그 디렉터리 경로
- `SNAPSHOT_ICEFALLDB_PATH`: `SNAPSHOT_STORE=icefalldb`일 때 snapshot icefalldb 로그 디렉터리 경로
- `SNAPSHOT_BITASK_PATH`: `SNAPSHOT_STORE=bitask`일 때 snapshot bitask append-only log 디렉터리 경로
- `SNAPSHOT_BITKV_RS_PATH`: `SNAPSHOT_STORE=bitkv_rs`일 때 snapshot bitkv-rs append-only log 디렉터리 경로
- `SNAPSHOT_CANDYSTORE_PATH`: `SNAPSHOT_STORE=candystore`일 때 snapshot candystore 디렉터리 경로
- `SNAPSHOT_CUENDILLAR_PATH`: `SNAPSHOT_STORE=cuendillar`일 때 snapshot cuendillar 루트 디렉터리 경로. 내부에 `wal/`, `sstable/` 디렉터리가 함께 생성된다
- `SNAPSHOT_JAMMDB_PATH`: `SNAPSHOT_STORE=jammdb`일 때 snapshot jammdb 파일 경로
- `SNAPSHOT_MACE_PATH`: `SNAPSHOT_STORE=mace`일 때 snapshot Mace 디렉터리 경로
- `SNAPSHOT_JANQL_PATH`: `SNAPSHOT_STORE=janql`일 때 snapshot janql WAL/SSTable 디렉터리 경로
- `SNAPSHOT_FJALL_PATH`: `SNAPSHOT_STORE=fjall`일 때 snapshot fjall DB 디렉터리 경로
- `SNAPSHOT_PERSY_PATH`: `SNAPSHOT_STORE=persy`일 때 snapshot persy 파일 경로
- `SNAPSHOT_PERSISTENT_KV_PATH`: `SNAPSHOT_STORE=persistent_kv`일 때 snapshot persistent-kv 디렉터리 경로
- `SNAPSHOT_NATIVE_DB_PATH`: `SNAPSHOT_STORE=native_db`일 때 snapshot native_db 파일 경로
- `SNAPSHOT_NEBARI_PATH`: `SNAPSHOT_STORE=nebari`일 때 snapshot nebari 디렉터리 경로
- `SNAPSHOT_NIKIDB_PATH`: `SNAPSHOT_STORE=nikidb`일 때 snapshot nikidb 파일 경로
- `SNAPSHOT_NODB_PATH`: `SNAPSHOT_STORE=nodb`일 때 snapshot nodb 파일 경로
- `SNAPSHOT_OKOFDB_PATH`: `SNAPSHOT_STORE=okofdb`일 때 snapshot okofdb key-per-file 디렉터리 경로
- `SNAPSHOT_PARITY_DB_PATH`: `SNAPSHOT_STORE=parity_db`일 때 snapshot parity-db 디렉터리 경로
- `SNAPSHOT_PICKLEDB_PATH`: `SNAPSHOT_STORE=pickledb`일 때 snapshot PickleDB 파일 경로
- `SNAPSHOT_RCASK_PATH`: `SNAPSHOT_STORE=rcask`일 때 snapshot RCask 세그먼트 디렉터리 경로
- `SNAPSHOT_MICROKV_PATH`: `SNAPSHOT_STORE=microkv`일 때 snapshot MicroKV base path. 실제 데이터 파일은 `<path>.kv`로 생성된다
- `SNAPSHOT_REDB_PATH`: `SNAPSHOT_STORE=redb`일 때 snapshot redb 파일 경로
- `SNAPSHOT_RSKEY_PATH`: `SNAPSHOT_STORE=rskey`일 때 snapshot rskey JSON hashmap 파일 경로
- `SNAPSHOT_READB_PATH`: `SNAPSHOT_STORE=readb`일 때 snapshot readb 디렉터리 경로
- `SNAPSHOT_RUSTLITE_PATH`: `SNAPSHOT_STORE=rustlite`일 때 snapshot rustlite 디렉터리 경로
- `SNAPSHOT_RUSTCASK_PATH`: `SNAPSHOT_STORE=rustcask`일 때 snapshot rustcask append-only Bitcask 디렉터리 경로
- `SNAPSHOT_RUSTY_LEVELDB_PATH`: `SNAPSHOT_STORE=rusty_leveldb`일 때 snapshot rusty-leveldb 디렉터리 경로
- `SNAPSHOT_CANOPYDB_PATH`: `SNAPSHOT_STORE=canopydb`일 때 snapshot canopydb 디렉터리 경로
- `SNAPSHOT_CAVES_PATH`: `SNAPSHOT_STORE=caves`일 때 snapshot caves key-per-file 디렉터리 경로
- `SNAPSHOT_CKYDB_PATH`: `SNAPSHOT_STORE=ckydb`일 때 snapshot ckydb 디렉터리 경로
- `SNAPSHOT_CREPEDB_PATH`: `SNAPSHOT_STORE=crepedb`일 때 snapshot CrepeDB redb 파일 경로
- `SNAPSHOT_SCDB_PATH`: `SNAPSHOT_STORE=scdb`일 때 snapshot scdb 디렉터리 경로
- `SNAPSHOT_SKV_PATH`: `SNAPSHOT_STORE=skv`일 때 snapshot skv base path. 실제 저장 파일은 `<path>.data`, `<path>.index`
- `SNAPSHOT_SURREALKV_PATH`: `SNAPSHOT_STORE=surrealkv`일 때 snapshot surrealkv B+tree 단일 파일 경로
- `SNAPSHOT_SLED_PATH`: `SNAPSHOT_STORE=sled`일 때 snapshot sled DB 디렉터리 경로
- `SNAPSHOT_RUSTBREAK_PATH`: `SNAPSHOT_STORE=rustbreak`일 때 snapshot rustbreak 단일 파일 경로
- `SNAPSHOT_YEDB_PATH`: `SNAPSHOT_STORE=yedb`일 때 snapshot yedb DB 디렉터리 경로
- `SNAPSHOT_BTREE_STORE_PATH`: `SNAPSHOT_STORE=btree_store`일 때 snapshot btree-store 단일 파일 경로
- `SNAPSHOT_SIAMESDB_PATH`: `SNAPSHOT_STORE=siamesedb`일 때 snapshot siamesedb DB 디렉터리 경로
- `SNAPSHOT_STRUCTSY_PATH`: `SNAPSHOT_STORE=structsy`일 때 snapshot structsy 단일 파일 경로
- `SNAPSHOT_ABYSSINIANDB_PATH`: `SNAPSHOT_STORE=abyssiniandb`일 때 snapshot abyssiniandb 단일 파일 경로
- `SNAPSHOT_AETERNUSDB_PATH`: `SNAPSHOT_STORE=aeternusdb`일 때 snapshot aeternusdb 디렉터리 경로
- `SNAPSHOT_THUNDERDB_PATH`: `SNAPSHOT_STORE=thunderdb`일 때 snapshot thunderdb 단일 파일 경로
- `SNAPSHOT_THETADB_PATH`: `SNAPSHOT_STORE=thetadb`일 때 snapshot thetadb 단일 파일 경로
- `SNAPSHOT_TINYBASE_PATH`: `SNAPSHOT_STORE=tinybase`일 때 snapshot tinybase sled 디렉터리 경로
- `SNAPSHOT_TINYDB_PATH`: `SNAPSHOT_STORE=tinydb`일 때 snapshot tinydb bincode 단일 파일 경로
- `SNAPSHOT_DBLITE_PATH`: `SNAPSHOT_STORE=dblite`일 때 snapshot dblite 단일 파일 경로
- `SNAPSHOT_DBLESS_PATH`: `SNAPSHOT_STORE=dbless`일 때 snapshot dbless 단일 파일 경로
- `SNAPSHOT_DB_RS_PATH`: `SNAPSHOT_STORE=db_rs`일 때 snapshot db-rs append-only 로그 디렉터리 경로
- `SNAPSHOT_DHARMADB_PATH`: `SNAPSHOT_STORE=dharmadb`일 때 snapshot dharmadb WAL/SSTable 디렉터리 경로
- `SNAPSHOT_SANAKIRJA_PATH`: `SNAPSHOT_STORE=sanakirja`일 때 snapshot sanakirja 단일 파일 경로
- `SNAPSHOT_SNAILDB_PATH`: `SNAPSHOT_STORE=snaildb`일 때 snapshot snaildb 디렉터리 경로
- `SNAPSHOT_TINYKV_PATH`: `SNAPSHOT_STORE=tinykv`일 때 snapshot tinykv JSON 파일 경로
- `SNAPSHOT_VSDB_PATH`: `SNAPSHOT_STORE=vsdb`일 때 snapshot vsdb handle metadata 디렉터리 경로. `store.meta.json`이 생성되며, 실제 keyspace는 upstream `vsdb`의 process-global base dir(`VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb`)을 따른다
- `SNAPSHOT_YAKV_PATH`: `SNAPSHOT_STORE=yakv`일 때 snapshot yakv 단일 파일 경로
- `SNAPSHOT_SABERDB_PATH`: `SNAPSHOT_STORE=saberdb`일 때 snapshot saberdb JSON 파일 경로
- `SNAPSHOT_SMOLLDB_PATH`: `SNAPSHOT_STORE=smolldb`일 때 snapshot SmollDB compressed 단일 파일 경로
- `SNAPSHOT_KSTONE_PATH`: `SNAPSHOT_STORE=kstone`일 때 snapshot Kstone WAL/SSTable LSM 디렉터리 경로
- `SNAPSHOT_FEOXDB_PATH`: `SNAPSHOT_STORE=feoxdb`일 때 snapshot FeOxDB 단일 파일 경로
- `SNAPSHOT_JSONDB_PATH`: `SNAPSHOT_STORE=jsondb`일 때 snapshot jsondb JSON 파일 경로
- `SNAPSHOT_KOPPERDB_PATH`: `SNAPSHOT_STORE=kopperdb`일 때 snapshot kopperdb 세그먼트 디렉터리 경로
- `SNAPSHOT_KV_PATH`: `SNAPSHOT_STORE=kv`일 때 snapshot kv sled 디렉터리 경로
- `SNAPSHOT_KOIT_PATH`: `SNAPSHOT_STORE=koit`일 때 snapshot koit JSON 파일 경로
- `SNAPSHOT_LITE_DB_PATH`: `SNAPSHOT_STORE=lite_db`일 때 snapshot LiteDb 디렉터리 경로
- `SNAPSHOT_LOG_KV_PATH`: `SNAPSHOT_STORE=log_kv`일 때 snapshot append-only 단일 파일 경로
- `SNAPSHOT_LSM_STORAGE_ENGINE_PATH`: `SNAPSHOT_STORE=lsm_storage_engine`일 때 snapshot lsm_storage_engine WAL/SSTable 디렉터리 경로
- `SNAPSHOT_MINDB_PATH`: `SNAPSHOT_STORE=mindb`일 때 snapshot Mindb WAL/SSTable 디렉터리 경로
- `SNAPSHOT_MMDB_PATH`: `SNAPSHOT_STORE=mmdb`일 때 snapshot MMDB WAL/SSTable 디렉터리 경로
- `SNAPSHOT_NANODB_PATH`: `SNAPSHOT_STORE=nanodb`일 때 snapshot NanoDB single JSON 파일 경로
- `SNAPSHOT_JFS_PATH`: `SNAPSHOT_STORE=jfs`일 때 snapshot jfs single-file JSON catalog 경로
- `SNAPSHOT_JSON_STORE_PATH`: `SNAPSHOT_STORE=json_store`일 때 snapshot json_store append-only JSON line catalog 경로
- `SNAPSHOT_S3_ENDPOINT`: `SNAPSHOT_STORE=s3`일 때 S3-compatible object storage endpoint
- `SNAPSHOT_S3_REGION`: S3 signing region
- `SNAPSHOT_S3_BUCKET`: snapshot object를 저장할 bucket 이름
- `SNAPSHOT_S3_PREFIX`: snapshot object key prefix. 기본값은 `snapshots/`
- `SNAPSHOT_S3_ACCESS_KEY_ID`: S3 access key id
- `SNAPSHOT_S3_SECRET_ACCESS_KEY`: S3 secret access key
- `SNAPSHOT_S3_SESSION_TOKEN`: optional session token
- `SNAPSHOT_S3_TIMEOUT_SECS`: S3 object storage HTTP timeout(초)
- `SNAPSHOT_S3_PATH_STYLE`: path-style addressing 사용 여부. 기본값은 `true`
- `SNAPSHOT_MANAGED_BASE_URL`: `SNAPSHOT_STORE=managed`일 때 external snapshot service base URL
- `SNAPSHOT_MANAGED_AUTH_TOKEN`: managed snapshot service에 보낼 optional Bearer 토큰
- `SNAPSHOT_MANAGED_TIMEOUT_SECS`: managed snapshot service HTTP timeout(초)
- `ROOM_LOCATOR`: `local`, `static`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR`: `noop`, `logging`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 파일을 저장하는 디렉터리이며, `ROOM_LOCATOR=file`은 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_SQLITE_PATH`: `ROOM_COORDINATOR=sqlite`일 때 lease row를 저장하는 SQLite DB 파일 경로이며, `ROOM_LOCATOR=sqlite`는 같은 DB를 읽는다
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease heartbeat 갱신 간격(초)
- `ROOM_COORDINATOR_LEASE_TTL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease 만료 TTL(초)
- `ROOM_COORDINATION_MANAGED_BASE_URL`: `ROOM_LOCATOR=managed` 또는 `ROOM_COORDINATOR=managed`일 때 외부 lease service base URL
- `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`: managed coordination service에 보낼 optional Bearer 토큰
- `ROOM_COORDINATION_MANAGED_TIMEOUT_SECS`: managed coordination service HTTP timeout(초)
- `NODE_ID`: 현재 collaboration node 식별자
- `NODE_BASE_URL`: 현재 collaboration node를 다른 노드에 안내할 때 사용할 canonical origin-only base URL. `ROOM_COORDINATOR=file|sqlite|managed` state와 conflict 응답의 `owner.base_url`에 반영된다.
- `ROOM_OWNER_HINTS_PATH`: `ROOM_LOCATOR=static`일 때 문서별 owner 힌트 JSON 파일 경로

## 현재 범위

- 단일 프로세스 room 관리
- room snapshot 저장/복구 및 idle eviction 정책
- 문서별 WebSocket 협업 세션 진입
- API/앱 상태/설정/에러 모듈 분리
- 테스트 가능한 앱 빌더 제공
- 기본 in-memory snapshot store와 로컬 file/flash_kv/grebedb/grumpydb/graus_db/highlandcows_isam/simple_db/docdb/eight/epoch_db/ferrumdb/rumdb/shorterdb/sqlite/heed/hightower_kv/hmdb/bitask/bitkv_rs/candystore/cuendillar/jammdb/mace/janql/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/jsondb/kv/koit/lite_db/log_kv/lsm_storage_engine/mindb/mmdb/nanodb/jfs/json_store/feoxdb, S3-compatible object storage, external managed snapshot store 지원
- config-driven room locator local/static/file/sqlite/managed 모드와 room coordinator dry-run logging/file/sqlite/managed state 모드 지원

## 비범위

- 데이터베이스 연동
- 문서 수정용 REST API
- 추가 vendor-specific database durability backend

현재 기본값은 여전히 단일 프로세스다. 다만 `SNAPSHOT_STORE=sqlite`와 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite`를 같은 shared SQLite DB 경로에 맞추면, lock-capable storage 위에서는 lease compare-and-swap과 snapshot 내구성을 함께 가져갈 수 있다. `SNAPSHOT_STORE=grebedb`, `SNAPSHOT_STORE=grumpydb`, `SNAPSHOT_STORE=graus_db`, `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=hmdb`, `SNAPSHOT_STORE=icefalldb`, `SNAPSHOT_STORE=bitask`, `SNAPSHOT_STORE=bitkv_rs`, `SNAPSHOT_STORE=candystore`, `SNAPSHOT_STORE=highlandcows_isam`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=mace`, `SNAPSHOT_STORE=janql`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=nebari`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=okofdb`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=rcask`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=rskey`, `SNAPSHOT_STORE=readb`, `SNAPSHOT_STORE=rustlite`, `SNAPSHOT_STORE=rustcask`, `SNAPSHOT_STORE=rusty_leveldb`, `SNAPSHOT_STORE=canopydb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=crepedb`, `SNAPSHOT_STORE=scdb`, `SNAPSHOT_STORE=surrealkv`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=thetadb`, `SNAPSHOT_STORE=tinybase`, `SNAPSHOT_STORE=tinydb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=db_rs`, `SNAPSHOT_STORE=dharmadb`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=yakv`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=smolldb`, `SNAPSHOT_STORE=kstone`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=kopperdb`, `SNAPSHOT_STORE=kv`, `SNAPSHOT_STORE=eight`, `SNAPSHOT_STORE=epoch_db`, `SNAPSHOT_STORE=ferrumdb`, `SNAPSHOT_STORE=rumdb`, `SNAPSHOT_STORE=koit`, `SNAPSHOT_STORE=lite_db`, `SNAPSHOT_STORE=log_kv`, `SNAPSHOT_STORE=lsm_storage_engine`, `SNAPSHOT_STORE=mindb`, `SNAPSHOT_STORE=mmdb`, `SNAPSHOT_STORE=nanodb`, `SNAPSHOT_STORE=jfs`, `SNAPSHOT_STORE=json_store`, `SNAPSHOT_STORE=simple_db`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=shorterdb`는 같은 `SnapshotStore` 경계를 vendor-specific embedded database durability로 확장해 로컬 durable restart 복구를 제공한다. `SNAPSHOT_STORE=s3`는 object key 단위 durability를 제공하고, `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 external lease service에 연결하고 `SNAPSHOT_STORE=managed`를 external snapshot service에 연결하면 ownership coordination plane과 snapshot durability plane을 shared SQLite 밖으로도 분리할 수 있다. 현재 저장소는 managed coordination + managed snapshot durability 조합까지 실제 multi-host handoff 회귀 테스트로 검증한다.

현재 `blocked` 상태는 실행 환경 차원의 commit/push/test 제한을 별도 관리하는 정도로 축소됐다. 반면 vendor-specific embedded database durability backend인 heed/hightower_kv/hmdb/icefalldb/bitask/bitkv_rs/candystore/highlandcows_isam/jammdb/mace/janql/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/jsondb/kopperdb/kv/eight/epoch_db/ferrumdb/rumdb/koit/lite_db/log_kv/lsm_storage_engine/mindb/mmdb/nanodb/jfs/json_store/simple_db/docdb/shorterdb, S3-compatible object storage durability backend, shared SQLite를 넘어서는 external durability backend 자체, managed-managed owner handoff rehearsal은 이제 회귀 테스트로 검증됐다.

## Embedded Snapshot Store Selection Guide

다음 기준은 `SNAPSHOT_STORE=grebedb|grumpydb|graus_db|heed|hightower_kv|hmdb|icefalldb|bitask|bitkv_rs|candystore|highlandcows_isam|jammdb|mace|janql|fjall|persy|persistent_kv|native_db|nebari|nikidb|nodb|okofdb|parity_db|pickledb|rcask|microkv|redb|rskey|readb|rustlite|rustcask|rusty_leveldb|canopydb|caves|ckydb|crepedb|scdb|skv|surrealkv|sled|rustbreak|yedb|btree_store|siamesedb|structsy|abyssiniandb|aeternusdb|thunderdb|thetadb|tinybase|dblite|dbless|db_rs|dharmadb|sanakirja|snaildb|tinykv|yakv|saberdb|smolldb|kstone|jsondb|kopperdb|kv|eight|epoch_db|rumdb|koit|lite_db|log_kv|lsm_storage_engine|mindb|mmdb|nanodb|jfs|json_store|simple_db|docdb|shorterdb` 중 어떤 embedded durability backend를 운영 기본값으로 둘지 고를 때 사용한다.

| 운영 목표 | 우선 후보 | 이유 |
| --- | --- | --- |
| 실제 multi-node owner handoff까지 같은 저장소에서 끝내기 | `sqlite` 또는 `managed` | embedded backend는 snapshot durability만 제공한다. authoritative lease CAS까지 묶으려면 `ROOM_LOCATOR`/`ROOM_COORDINATOR`와 같은 coordination plane을 함께 제공하는 `sqlite` 또는 `managed`가 필요하다. |
| 단일 노드 재시작 복구를 가장 단순하게 운영하기 | `file`, `agdb`, `jammdb`, `persy`, `native_db`, `nikidb`, `nodb`, `redb`, `crepedb`, `rskey`, `rustbreak`, `btree_store`, `structsy`, `abyssiniandb`, `surrealkv`, `thunderdb`, `thetadb`, `tinydb`, `dblite`, `dbless`, `sanakirja`, `tinykv`, `yakv`, `saberdb`, `smolldb`, `kstone`, `jsondb`, `koit`, `lite_db`, `log_kv`, `lsm_storage_engine`, `mmdb`, `nanodb`, `jfs`, `json_store`, `simple_db`, `docdb`, `rumdb`, `rcask` | 단일 path 또는 단일 directory 기준 백업/복사 절차를 잡기 쉽다. 운영자가 파일 또는 로그 디렉터리 단위 스냅샷, 교체, 롤백을 직접 다루기 편하다. |
| 디렉터리 단위 엔진과 내부 keyspace/map 구조를 유지하기 | `heed`, `hightower_kv`, `hmdb`, `bitask`, `bitkv_rs`, `candystore`, `cuendillar`, `highlandcows_isam`, `epoch_db`, `rumdb`, `fjall`, `parity_db`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `ckydb`, `scdb`, `sled`, `yedb`, `siamesedb`, `snaildb`, `shorterdb`, `tinybase`, `db_rs` | 엔진이 디렉터리 아래 여러 파일과 내부 카탈로그를 관리하거나 index/log/data 파일을 분리한다. `cuendillar`도 `wal/`, `sstable/` 하위 디렉터리를 함께 관리하므로 파일 하나만 교체하는 운영보다 디렉터리 전체 백업/restore 절차가 자연스럽다. |
| 사람이 직접 payload를 확인하거나 임시 복구하기 쉽게 유지하기 | `file`, `pickledb`, `rcask`, `microkv`, `rskey`, `docdb`, `nanodb`, `json_store` | 구현이 JSON 또는 단순 key-value 파일 경계에 가깝다. 반면 엔진 주도 포맷보다 대용량 catalog scan과 payload 손상 대응은 더 보수적으로 봐야 한다. |
| catalog 한 파일 손상이 전체 startup 복구 실패로 바로 번지는 경로를 피하기 | `hightower_kv`, `jammdb`, `persy`, `native_db`, `redb`, `btree_store`, `siamesedb`, `abyssiniandb`, `ckydb`, `crepedb`, `scdb`, `skv`, `surrealkv`, `thunderdb`, `sanakirja`, `snaildb`, `shorterdb` | 현재 구현 기준으로 corrupt entry는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛴다. `hmdb`, `rustbreak`, `tinykv`, `yakv`, `saberdb`, `docdb`는 startup 시 로그/파일 전체 역직렬화에 더 의존하므로 운영 기본값으로 둘 때 별도 주의가 필요하다. |
| pure-Rust/no-bindgen/no-native-conflict 제약을 현재 빌드 그래프에서 그대로 유지하기 | `agdb`, `grumpydb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `sanakirja`, `snaildb`, `tinykv`, `yakv`, `saberdb`, `smolldb`, `kstone`, `jsondb`, `kv`, `eight`, `epoch_db`, `ferrumdb`, `rumdb`, `koit`, `lite_db`, `log_kv`, `lsm_storage_engine`, `mmdb`, `nanodb`, `jfs`, `json_store`, `simple_db`, `docdb`, `shorterdb`, `highlandcows_isam`, `nikidb`, `nodb`, `persistent_kv`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `scdb`, `skv`, `surrealkv`, `rskey`, `hightower_kv`, `hmdb`, `icefalldb`, `bitask`, `bitkv_rs`, `candystore`, `nebari`, `rcask` | 최근 추가 backend가 이 제약을 실제 landed change로 통과했다. 같은 조건의 추가 후보를 검토할 때도 이 기준선을 먼저 보고, native `links` 충돌이나 bindgen 의존성이 생기면 제외한다. |

운영 중 backend를 바꿔야 할 때는 아래 매트릭스로 실제 파일 단위, payload 가시성, 손상 격리 특성을 먼저 비교한다.

| Backend | 저장 단위 | 운영자 payload 가시성 | 손상/복구 주의점 | 제약 메모 |
| --- | --- | --- | --- | --- |
| `file` | 문서별 JSON 파일 | 가장 높음 | 파일 하나 손상 시 해당 문서만 직접 격리 가능 | baseline filesystem store |
| `grumpydb` | 디렉터리 page/B+Tree object store | 중간 | `data.db`와 `index.db` 파일 세트에 UUID key와 bytes payload를 저장하고 full range scan으로 catalog를 복구한다. WAL 모듈은 아직 upstream roadmap 단계라 crash-consistency는 flush와 directory-level backup/restore로 보수적으로 검증해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `graus_db` | 디렉터리 append-only log store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 GrausDb log에 저장한다. write 뒤 DB handle을 재오픈해 buffered writer flush와 startup replay 경계를 회귀 테스트로 고정한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `mmdb` | 디렉터리 WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload key와 explicit `__catalog__` key를 write batch로 함께 저장하고 sync write 뒤 flush한다. 엔진 디렉터리 전체가 복구 단위라 directory-level backup/restore와 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nanodb` | 단일 JSON 파일 | 높음 | root object의 `doc_id -> persisted snapshot JSON` entry를 save/delete 뒤 whole-file write로 반영한다. 파일 단위 payload 점검은 쉽지만, 전체 파일 parse에 의존하므로 file corruption 시 startup 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `heed` | 디렉터리 + LMDB data file | 낮음 | 엔진 파일 단위 백업이 필요하고 수동 entry 복구는 어렵다 | mmap 기반, pure-Rust baseline에는 포함하지 않음 |
| `hightower_kv` | 디렉터리 + log-structured segments/snapshots | 낮음 | `snapshot:<doc_id>` prefix scan으로 catalog를 복구하므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `hmdb` | 디렉터리 + append-only bincode log | 낮음 | schema별 단일 로그 파일 replay로 catalog를 복구한다. tail truncation은 incomplete write로 흡수할 수 있지만, 중간 구간 손상이나 스키마 불일치는 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `icefalldb` | 디렉터리 + append-only `rsdb.log` | 낮음 | 공개 delete/iterator API가 없어 `doc_id` tombstone과 explicit `__catalog__` key를 함께 유지한다. restart recovery는 append-only log replay에 의존하므로 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitask` | 디렉터리 + append-only active/immutable logs | 낮음 | explicit `__catalog__` key를 같은 log에 유지한다. startup에는 log replay로 keydir를 재구축하고, writer lock이 단일 프로세스만 허용되므로 shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitkv_rs` | 디렉터리 + append-only Bitcask-style data files | 낮음 | `doc_id -> persisted snapshot JSON` key-value를 sync write로 저장하고 startup에는 log replay로 in-memory index를 복구한다. writer lock이 단일 프로세스만 허용되므로 shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `candystore` | 디렉터리 + append-only data/log/index files | 낮음 | large payload는 `set_big/get_big`로 저장하고 `__catalog__` key를 별도로 유지한다. `flush`와 `checkpoint`로 durable cursor를 전진시키므로 엔진 디렉터리 전체 백업이 필요하고, payload는 binary value라 수동 수정 대신 회귀 테스트 기반 복구가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jammdb` | 단일 파일 | 낮음 | bucket 내부 key는 분리되지만 payload는 binary라 수동 복구가 어렵다 | single-file backup에 유리 |
| `fjall` | 디렉터리 keyspace | 낮음 | LSM directory 전체를 함께 백업해야 한다 | directory-backed engine |
| `persy` | 단일 파일 + index | 낮음 | entry 단위 skip은 가능하지만 index 일관성 검증이 필요하다 | single-file engine |
| `persistent_kv` | 디렉터리 + WAL/snapshot set | 낮음 | snapshot 디렉터리 전체와 WAL/shard 파일을 함께 백업해야 하고, payload는 binary value라 수동 수정보다 재시작 복구 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `native_db` | 단일 파일 | 낮음 | primary-key catalog라 payload 직접 점검은 어렵다 | single-file engine |
| `nebari` | 디렉터리 + append-only tree store | 낮음 | `snapshots` tree range scan으로 catalog를 복구하므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nikidb` | 단일 파일 B+tree bucket store | 낮음 | explicit `__catalog__` key와 문서 payload가 같은 B+tree file에 함께 저장된다. 수동 payload inspection은 어렵지만 bucket upsert와 single-file backup 절차는 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nodb` | 단일 파일 DB | 중간 | map 전체를 dump/rename 경계로 다시 쓰고 reopen 시 전체 load에 의존하므로 file corruption 시 startup 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `okofdb` | 디렉터리 + key-per-file | 높음 | `doc_<uuid_simple>` key마다 파일이 분리돼 payload inspection과 부분 백업은 쉽지만, crate가 direct file overwrite를 사용하므로 crash-consistency는 디렉터리 단위 backup/restore와 회귀 테스트로 보완하는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `parity_db` | 디렉터리 column store | 낮음 | ordered column 전체를 묶어 관리해야 한다 | directory-backed engine |
| `pickledb` | 단일 JSON 유사 DB 파일 | 높음 | 사람이 읽기 쉽지만 대용량 catalog에서는 scan 비용을 더 보수적으로 본다 | text-oriented store |
| `rcask` | 디렉터리 + append-only log segments | 중간 | `doc_id` payload와 explicit `__catalog__` key를 UTF-8 JSON string으로 저장하고, 공개 delete API가 없어 tombstone string으로 삭제를 가린다. RCask 디렉터리 전체를 backup/restore해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `microkv` | 단일 `.kv` 파일 | 중간 | key-value 구조는 단순하지만 payload는 binary 직렬화라 완전 수동 복구엔 한계가 있다 | simple local KV |
| `redb` | 단일 파일 | 낮음 | tree 내부 payload는 직접 읽기 어렵지만 entry skip 전략과 잘 맞는다 | single-file engine |
| `rskey` | 단일 JSON hashmap 파일 | 높음 | store 전체를 한 파일로 다시 쓰므로 파일 손상이 startup 전체 복구 실패로 이어질 수 있다. 대신 `doc_id -> persisted snapshot JSON string` 구조라 수동 점검과 부분 복구는 쉽다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `readb` | 디렉터리 + append-only data/index | 낮음 | `__catalog__` key가 누락되면 문서 목록 복구가 좁아지므로 catalog key와 data 파일을 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kv` | 디렉터리 + sled tree keyspace | 낮음 | `snapshots` bucket의 `doc_id` key를 full scan해 catalog를 복구한다. payload는 JSON codec으로 직렬화되지만 engine 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `epoch_db` | 디렉터리 + sled-backed multi-tree store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 JSON string으로 저장한다. payload inspection보다 engine 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rumdb` | 디렉터리 + append-only Bitcask-style log set | 낮음 | `doc_id` key와 explicit `__catalog__` key를 append-only log에 저장한다. startup은 전체 log replay로 keydir를 복구하므로 directory 전체 백업/restore와 replay 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rustlite` | 디렉터리 + WAL/SSTable engine | 낮음 | `__catalog__` key와 engine 디렉터리를 함께 백업해야 catalog 복구 경로가 유지된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rustcask` | 디렉터리 + append-only Bitcask data/hint files | 낮음 | `doc_id` key와 explicit `__catalog__` key를 같은 append-only log에 저장한다. sync mode를 켜서 각 write를 fsync하지만, 운영 백업은 여전히 rustcask 디렉터리 전체를 기준으로 잡는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rusty_leveldb` | 디렉터리 + LevelDB keyspace/log/manifest | 낮음 | `doc_id -> persisted snapshot JSON` key-value를 저장하고, document catalog는 same keyspace full scan으로 복구한다. 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `canopydb` | 디렉터리 + transactional tree/WAL | 낮음 | `snapshots` tree iter scan으로 catalog를 복구하므로 engine 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `caves` | 디렉터리 + key-per-file | 높음 | key마다 별도 파일이라 payload 확인은 쉽지만, crate caveat상 `set/delete` 뒤 매번 sync를 보장하지 않으므로 crash-consistency는 운영 백업/복구 절차로 보완해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ckydb` | 디렉터리 + index/log/data files | 낮음 | `__catalog__` key와 ckydb 디렉터리 전체를 함께 백업해야 한다. payload는 base64 문자열이라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `crepedb` | 단일 redb 파일 | 낮음 | CrepeDB basic table에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장한다. redb 파일 단위 백업/restore와 회귀 테스트 기반 검증이 기본 절차다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `scdb` | 디렉터리 + `dump.scdb` | 낮음 | `__catalog__` key와 scdb 디렉터리 전체를 함께 백업해야 한다. payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `skv` | data/index 파일 쌍 | 낮음 | `doc_id` key와 explicit `__catalog__` key를 `<path>.data`/`<path>.index` 파일 쌍에 저장한다. 두 파일을 항상 함께 백업/restore해야 restart 복구 경계가 유지된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `surrealkv` | 단일 파일 B+tree | 낮음 | full scan catalog는 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `sled` | 디렉터리 DB | 낮음 | 엔진 디렉터리 전체 백업과 restore가 기본 절차다 | directory-backed engine |
| `rustbreak` | 단일 파일 catalog | 중간 | catalog 전체 역직렬화 실패가 startup 실패로 이어질 수 있어 사전 백업 검증이 중요하다 | single-file but whole-file risk |
| `yedb` | 디렉터리 + per-key files | 중간 | key 파일이 나뉘어 있어 수동 탐색은 가능하지만 directory 전체 일관성을 같이 봐야 한다 | directory-backed text-friendly KV |
| `btree_store` | 단일 파일 | 낮음 | btree bucket은 binary지만 entry 단위 skip 전략과 잘 맞는다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `siamesedb` | 디렉터리 map store | 낮음 | map key는 분리되지만 engine iterator 특성 때문에 catalog key 보조 관리가 필요하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `structsy` | 단일 파일 record store | 중간 | record scan은 단순하지만 payload는 struct record라 임의 수정 대신 export/import 절차가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `abyssiniandb` | 단일 파일 key-value store | 낮음 | key/value는 단순하지만 payload와 catalog 모두 binary value라 수동 복구보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `aeternusdb` | 디렉터리 + WAL/SSTable LSM engine | 낮음 | payload와 catalog 모두 binary value라 수동 수정보다 entry skip 기반 대응이 안전하고, `__catalog__` key를 엔진 디렉터리와 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `thunderdb` | 단일 파일 transactional KV | 낮음 | bucket iter scan은 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `thetadb` | 단일 파일 B+tree KV | 낮음 | cursor full scan으로 catalog를 복구하기 쉽지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `tinybase` | 디렉터리 + sled trees / typed table index | 낮음 | `doc_id` secondary index와 constant catalog index를 함께 재구성하므로 sled 디렉터리 전체 백업이 필요하고, bincode payload/인덱스 손상 시 startup/query 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `tinydb` | 단일 bincode dump 파일 | 낮음 | `doc_id`별 record를 HashSet에 보관하고 save/delete마다 전체 DB를 다시 dump한다. 단일 파일 백업은 단순하지만 payload가 bincode라 수동 점검보다 회귀 테스트 기반 복구가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dblite` | 단일 파일 append/reuse KV | 중간 | key index는 reopen 시 파일 전체 scan으로 재구성되고 file-level lock에 의존하므로, 단일 파일 백업은 단순하지만 partial file corruption 시 재구성 실패 가능성을 염두에 둬야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dbless` | 단일 파일 typed table store | 낮음 | redb-backed typed table이라 수동 payload inspection은 어렵지만 named table key scan은 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `db_rs` | 디렉터리 + append-only typed table log | 낮음 | `LookupTable<String, PersistedSnapshot>`가 append-only bincode log를 replay해 catalog를 재구성하므로 디렉터리 전체 백업이 필요하고, payload는 binary라 수동 수정 대신 회귀 테스트 기반 복구가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dharmadb` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 같은 keyspace에 저장한다. upstream DB 인스턴스가 비-Send라 adapter가 전용 worker thread로 접근을 직렬화하고, 운영자는 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `sanakirja` | 단일 파일 copy-on-write B-tree | 낮음 | full scan catalog는 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `snaildb` | 디렉터리 + WAL/SSTable LSM engine | 낮음 | payload와 `__catalog__` key를 함께 flush한 뒤 catalog를 읽으므로 엔진 디렉터리 전체를 함께 백업해야 하고, 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `shorterdb` | 디렉터리 + WAL/SST LSM engine | 낮음 | 공개 key iterator가 없어 `__catalog__` key를 별도로 유지한다. WAL replay와 background flush를 전제로 하므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `tinykv` | 단일 JSON 파일 store | 중간 | payload 가시성은 가장 높지만 whole-file rewrite와 전체 JSON 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `yakv` | 단일 B-Tree 파일 | 낮음 | `snapshot:<doc_id>` key를 직접 저장하고 full scan으로 catalog를 복구한다. payload는 binary value이고 파일 전체 무결성에 의존하므로 수동 수정 대신 whole-file backup/restore와 회귀 테스트가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `saberdb` | 단일 pretty JSON 파일 store | 중간 | atomic temp+rename은 단순하지만 catalog 전체를 pretty JSON으로 다시 쓰고 startup 시 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `smolldb` | 단일 compressed 파일 store | 낮음 | in-memory map을 zlib-compatible compressed 파일로 temp+rename 저장한다. 전체 파일 load/rewrite 경계라 단일 노드 재시작 복구에 맞고, corruption 시 startup 전체 복구 실패가 될 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kstone` | 디렉터리 + WAL/SSTable LSM store | 낮음 | Kstone item의 binary payload에 `snapshot:<doc_id>` 값과 explicit `__catalog__` key를 저장하고 save/delete 뒤 flush한다. 엔진 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `feoxdb` | 단일 FeOxDB 파일 append event store | 중간 | mutable same-key update replay를 피하기 위해 `snapshot:<doc_id>:<timestamp>:<event_id>` immutable event key와 tombstone event만 쓴다. range scan으로 최신 event를 복구하며 기본 jemalloc feature는 끄고 `system-alloc`으로 연결한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jsondb` | 단일 versioned pretty JSON 파일 store | 중간 | write guard drop마다 whole-file pretty JSON rewrite와 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `koit` | 단일 structured JSON 파일 store | 중간 | 전체 catalog를 메모리에 로드한 뒤 save마다 whole-file rewrite와 sync를 수행하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lite_db` | 디렉터리 + append-only data files | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 sync write로 저장한다. file lock이 단일 writer를 전제하므로 shared multi-writer durability나 authoritative coordination plane으로는 쓰지 않는 것이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lsm_storage_engine` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 WAL-first engine에 저장하고 save/delete 뒤 flush로 복구 경계를 고정한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ferrumdb` | 단일 append-only log 파일 store | 낮음 | JSON value를 append-only log에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key로 저장하고 `FsyncPolicy::Always`로 write마다 sync한다. compaction 전까지 로그가 증가하므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `mindb` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 같은 keyspace에 저장하고 save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. adapter는 reopen point index가 비어 있는 경우 upstream `RecoveryManager`로 WAL을 재생해 catalog/snapshot을 복구한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | no-bindgen/no-new-native-conflict 기준선 |
| `jfs` | 단일 JSON object store | 높음 | single-file catalog를 temp+rename으로 교체해 각 `doc_id` JSON object를 저장한다. payload inspection은 쉽지만 whole-file parse에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `json_store` | 단일 append-only JSON line 파일 store | 높음 | key별 최신 line offset을 메모리 인덱스로 유지하므로 payload inspection은 쉽지만, compaction 없이는 append log가 계속 커지고 startup catalog rebuild는 전체 파일 replay에 의존한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `simple_db` | 단일 line-oriented text 파일 | 중간 | `doc_id:base64(payload)` 라인 전체를 다시 쓰므로 파일 단위 백업은 단순하지만 partial rewrite 시 최근 쓰기 일부가 유실될 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `docdb` | 단일 JSON 파일 store | 중간 | `doc_id -> persisted snapshot` map 전체를 temp+rename으로 다시 쓰므로 단일 파일 백업은 단순하지만 파일 전체 역직렬화 실패가 startup 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |

- `skv`는 data/index 파일 쌍을 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 최신 기준선이다.
- `btree_store`는 single-file backup/restore 절차를 원하는 경우의 기준선이다.
- `structsy`는 single-file struct 레코드 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `abyssiniandb`는 single-file key-value 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `aeternusdb`는 WAL/SSTable 기반 LSM 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `hightower_kv`는 prefix-indexed log-structured 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `hmdb`는 append-only bincode 로그 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `icefalldb`는 append-only `rsdb.log` 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `surrealkv`는 single-file B+tree 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `thunderdb`는 single-file transactional key-value 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `thetadb`는 single-file B+tree key-value 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `tinybase`는 sled-backed typed table/secondary index 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `db_rs`는 append-only typed table 로그 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `dharmadb`는 WAL/SSTable LSM 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `dblite`는 single-file append/reuse key-value 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `dbless`는 redb-backed single-file typed table 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `sanakirja`는 single-file copy-on-write B-tree 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `snaildb`는 WAL/SSTable 기반 LSM 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `shorterdb`는 WAL/SST 기반 LSM 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `nikidb`는 single-file B+tree bucket 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `nodb`는 single-file dump/rename 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `okofdb`는 key-per-file 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `tinykv`는 human-readable single-file JSON 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `yakv`는 single-file B-Tree 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `saberdb`는 atomic temp+rename pretty JSON 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `smolldb`는 in-memory key-value map을 compressed single-file backup으로 flush하면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `kstone`은 WAL/SSTable LSM 디렉터리 저장소를 쓰면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `jsondb`는 schema-versioned pretty JSON 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `rcask`는 append-only segment log 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `lite_db`는 LiteDb append-only 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `lsm_storage_engine`은 zero-dependency WAL/SSTable LSM 디렉터리 저장소를 쓰면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `mindb`는 WAL/SSTable LSM 디렉터리 저장소를 쓰며 현재 빌드 그래프에서 no-bindgen/no-new-native-conflict 조건을 유지한 추가 기준선이다. 다만 upstream이 `zstd`를 의존하므로 pure-Rust baseline에는 포함하지 않는다.
- `koit`는 async whole-file structured JSON 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `jfs`는 single-file JSON object 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `json_store`는 append-only single-file JSON line 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `grumpydb`는 page/B+Tree 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `graus_db`는 append-only log 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `blockbucket`은 single-file raw bytes bucket 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `epoch_db`는 sled-backed multi-tree 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `rumdb`는 append-only Bitcask-style 로그 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `rustcask`는 append-only Bitcask 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `persistent_kv`는 snapshot set + WAL 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `simple_db`는 line-oriented single-file 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `docdb`는 auto-dump single-file JSON 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `rskey`는 single-file JSON hashmap 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `readb`는 append-only 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `rustlite`는 WAL/SSTable 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `canopydb`는 transactional tree/WAL 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `caves`는 key-per-file 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `ckydb`는 memory-first 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `scdb`는 localStorage-style 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `file`/`pickledb`/`rcask`/`microkv`는 운영자가 payload를 직접 읽기 쉽지만, authoritative coordination plane을 대체하지는 못한다.
- embedded backend를 어떤 것으로 고르더라도 `ROOM_LOCATOR=file|static`은 rehearsal/best-effort 경계로만 보고, 실제 handoff가 필요하면 `sqlite` 또는 `managed` coordination을 함께 사용한다.

`ROOM_LOCATOR=static`은 외부 coordinator를 대체하지 않는다. 대신 운영자가 문서별 owner 힌트를 선언해 현재 노드 비소유 문서를 조기에 거절하고, 응답 JSON의 `owner.node_id` / optional `owner.base_url` 및 대응 헤더로 upstream 라우팅 결정을 돕는 용도다. 힌트에 없는 문서는 현재 노드 소유로 간주한다.

`ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active room lease state를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `FileRoomCoordinator`가 같은 디렉터리에 남긴 state를 소비하는 best-effort resolver이며, `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 upstream 라우팅 결정을 도울 수 있다. stale owner 판단은 file mtime이 아니라 persisted `expires_at`만 기준으로 한다.

`ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에서 active lease row를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `SqliteRoomCoordinator`가 같은 DB에 기록한 lease를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

`ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에서 `GET /v1/leases/:doc_id`를 조회해 현재 노드 비소유 문서를 거절한다. 이 모드는 `ManagedRoomCoordinator`가 같은 service에 기록한 canonical lease record를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

## 향후 확장 방향

- provider awareness payload 연동
- 외부 저장소 adapter 추가
- provider / frontend editor 연동 계약 고도화
- 추가 vendor-specific database durability backend

## Snapshot Restore / Eviction Policy

- 문서 생성 시 초기 snapshot을 저장하고 active room을 메모리에 등록한다.
- `GET /api/documents`는 active room이 없어도 snapshot store에 남아 있는 문서를 카탈로그로 반환한다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 먼저 `RoomLocator`로 현재 노드 ownership을 확인한 뒤, active room이 없으면 snapshot store에서 room을 on-demand로 복구한다.
- WebSocket 세션이 종료될 때마다 room의 active session 수를 감소시키고, 마지막 세션이 닫히면 최신 snapshot을 저장한 뒤 room을 메모리에서 제거한다.
- 문서가 삭제된 경우에는 snapshot과 active room을 함께 제거한다. 활성 WebSocket 세션이 남아 있으면 삭제를 거절하고 `409 conflict`를 반환한다.
- `SNAPSHOT_STORE=file`일 때 손상된 snapshot 파일은 startup hydrate와 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛴다. 해당 문서를 직접 복구하려고 로드하면 여전히 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=file` 저장은 같은 디렉터리의 임시 파일 작성 후 `rename`으로 마무리해, 저장 도중 프로세스가 중단돼도 마지막 정상 snapshot을 바로 덮어쓰지 않도록 한다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되고, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 정리한다.
- `SNAPSHOT_STORE=file`이면 snapshot과 문서 토큰이 `SNAPSHOT_DIR/<doc_id>.json`에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 해당 디렉터리에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=agdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_AGDB_PATH` agdb 단일 파일의 `snapshot:<doc_id>` alias node에 JSON payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 agdb alias catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=grebedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_GREBEDB_PATH` 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. payload와 catalog는 같은 `flush()` 경계에서 함께 반영돼 기본 local ownership 모드에서는 앱 시작 시 grebedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=grumpydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_GRUMPYDB_PATH` 디렉터리의 GrumpyDB UUID key와 bytes payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 full range scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=graus_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_GRAUS_DB_PATH` 디렉터리의 GrausDb append-only log store에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay catalog로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=sqlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_SQLITE_PATH` SQLite DB의 `snapshots` 테이블에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 DB catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=heed`이면 snapshot과 문서 토큰이 `SNAPSHOT_HEED_PATH` heed LMDB 디렉터리의 `snapshots` database에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 heed catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=hightower_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_HIGHTOWER_KV_PATH` hightower-kv 디렉터리의 `snapshot:<doc_id>` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 prefix scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=bitask`이면 snapshot과 문서 토큰이 `SNAPSHOT_BITASK_PATH` bitask append-only log 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay로 keydir를 재구축한 뒤 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=bitkv_rs`이면 snapshot과 문서 토큰이 `SNAPSHOT_BITKV_RS_PATH` bitkv-rs append-only log 디렉터리의 `doc_id -> persisted snapshot JSON` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay로 in-memory index를 재구축한 뒤 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=candystore`이면 snapshot과 문서 토큰이 `SNAPSHOT_CANDYSTORE_PATH` candystore 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. large payload는 `set_big/get_big` 경로를 사용하고, save/delete 뒤 `flush`와 `checkpoint`를 수행해 기본 local ownership 모드에서는 앱 시작 시 candystore catalog에서 room을 eager hydrate하며 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=jammdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JAMMDB_PATH` jammdb 파일의 `snapshots` bucket에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 jammdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=mace`이면 snapshot과 문서 토큰이 `SNAPSHOT_MACE_PATH` Mace 디렉터리의 `snapshots` bucket에 `doc_id -> persisted snapshot JSON` 엔트리로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 Mace catalog key에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=janql`이면 snapshot과 문서 토큰이 `SNAPSHOT_JANQL_PATH` janql WAL/SSTable 디렉터리의 `doc_id -> persisted snapshot JSON` 엔트리와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 janql catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=fjall`이면 snapshot과 문서 토큰이 `SNAPSHOT_FJALL_PATH` fjall DB 디렉터리의 `snapshots` keyspace에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 fjall catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=persy`이면 snapshot과 문서 토큰이 `SNAPSHOT_PERSY_PATH` persy 파일의 `snapshots` segment와 `snapshots_by_doc_id` index에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 persy catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=persistent_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_PERSISTENT_KV_PATH` persistent-kv 디렉터리의 snapshot set/WAL에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=native_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_NATIVE_DB_PATH` native_db 파일의 primary-key catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 native_db catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=nikidb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NIKIDB_PATH` nikidb 단일 파일의 `snapshots` bucket과 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 nikidb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=parity_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_PARITY_DB_PATH` parity-db 디렉터리의 ordered `snapshots` column에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 parity-db BTree catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=redb`이면 snapshot과 문서 토큰이 `SNAPSHOT_REDB_PATH` redb 파일의 `snapshots` 테이블에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 redb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rskey`이면 snapshot과 문서 토큰이 `SNAPSHOT_RSKEY_PATH` rskey JSON hashmap 파일의 `doc_id -> persisted snapshot JSON string` 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rskey catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=readb`이면 snapshot과 문서 토큰이 `SNAPSHOT_READB_PATH` readb 디렉터리의 append-only data file과 index catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 readb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTLITE_PATH` rustlite 디렉터리의 WAL/SSTable engine과 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rustlite catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustcask`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTCASK_PATH` rustcask 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. sync mode를 켜서 각 write 뒤 fsync를 보장하며, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하고 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=rusty_leveldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTY_LEVELDB_PATH` rusty-leveldb 디렉터리의 LevelDB keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same keyspace full scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=canopydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CANOPYDB_PATH` canopydb 디렉터리의 `snapshots` tree와 transactional WAL/data file에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 canopydb tree catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=caves`이면 snapshot과 문서 토큰이 `SNAPSHOT_CAVES_PATH` 디렉터리의 `<doc_id>` key-per-file 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 directory scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=ckydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CKYDB_PATH` ckydb 디렉터리의 explicit `__catalog__` key와 key-value 엔트리에 저장된다. payload와 catalog는 delimiter-safe write를 위해 base64 문자열로 저장되며, 기본 local ownership 모드에서는 앱 시작 시 ckydb catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=crepedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CREPEDB_PATH` CrepeDB redb 파일의 basic `snapshots` table에 저장된다. payload는 `snapshot:<doc_id>` key에, 문서 목록은 explicit `__catalog__` key에 저장되며, 기본 local ownership 모드에서는 앱 시작 시 CrepeDB catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=scdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SCDB_PATH` scdb 디렉터리의 explicit `__catalog__` key와 `doc_id` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 scdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=skv`이면 snapshot과 문서 토큰이 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data`와 `<path>.index` 파일 쌍의 `doc_id` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=surrealkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_SURREALKV_PATH` surrealkv B+tree 단일 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 surrealkv full scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=pickledb`이면 snapshot과 문서 토큰이 `SNAPSHOT_PICKLEDB_PATH` PickleDB 파일의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 PickleDB catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rcask`이면 snapshot과 문서 토큰이 `SNAPSHOT_RCASK_PATH` RCask append-only log segment 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 JSON string으로 저장된다. 공개 delete API가 없어 tombstone string으로 삭제를 가리며, 기본 local ownership 모드에서는 앱 시작 시 RCask catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=microkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_MICROKV_PATH` base path에 대응하는 MicroKV 파일 `<path>.kv`의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 MicroKV catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=sled`이면 snapshot과 문서 토큰이 `SNAPSHOT_SLED_PATH` sled DB 디렉터리의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 sled catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustbreak`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTBREAK_PATH` rustbreak 단일 파일 catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rustbreak catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_YEDB_PATH` yedb 디렉터리의 `snapshots/<doc_id>` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=btree_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_BTREE_STORE_PATH` btree-store 단일 파일의 `snapshots` bucket key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 btree-store catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=siamesedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SIAMESDB_PATH` siamesedb 디렉터리의 `snapshots` map에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 siamesedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=structsy`이면 snapshot과 문서 토큰이 `SNAPSHOT_STRUCTSY_PATH` structsy 단일 파일의 persisted record catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 structsy catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=abyssiniandb`이면 snapshot과 문서 토큰이 `SNAPSHOT_ABYSSINIANDB_PATH` abyssiniandb 단일 파일의 `snapshots` map key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 abyssiniandb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=thunderdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_THUNDERDB_PATH` thunderdb 단일 파일의 `snapshots` bucket key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 thunderdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=thetadb`이면 snapshot과 문서 토큰이 `SNAPSHOT_THETADB_PATH` thetadb 단일 파일의 raw `doc_id` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 thetadb cursor full scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=tinybase`이면 snapshot과 문서 토큰이 `SNAPSHOT_TINYBASE_PATH` tinybase sled 디렉터리의 typed table record와 `doc_id`/catalog secondary index에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 tinybase catalog query로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=tinydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_TINYDB_PATH` tinydb bincode dump 파일의 `doc_id` keyed record에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 tinydb dump를 load해 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=dblite`이면 snapshot과 문서 토큰이 `SNAPSHOT_DBLITE_PATH` dblite 단일 파일의 string key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 dblite key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=dbless`이면 snapshot과 문서 토큰이 `SNAPSHOT_DBLESS_PATH` dbless 단일 파일의 typed table 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=db_rs`이면 snapshot과 문서 토큰이 `SNAPSHOT_DB_RS_PATH` 디렉터리 아래 db-rs append-only typed table log의 `LookupTable<String, PersistedSnapshot>` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same log replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=dharmadb`이면 snapshot과 문서 토큰이 `SNAPSHOT_DHARMADB_PATH` dharmadb WAL/SSTable 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. upstream DB 인스턴스가 비-Send라 adapter는 전용 worker thread에서 접근을 직렬화하고, 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate한다. distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=sanakirja`이면 snapshot과 문서 토큰이 `SNAPSHOT_SANAKIRJA_PATH` sanakirja 단일 파일 copy-on-write B-tree의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 sanakirja full scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=snaildb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SNAILDB_PATH` snaildb 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 snaildb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=tinykv`이면 snapshot과 문서 토큰이 `SNAPSHOT_TINYKV_PATH` tinykv JSON 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 tinykv key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=vsdb`이면 `SNAPSHOT_VSDB_PATH/store.meta.json`에 store handle metadata가 저장되고, 실제 snapshot payload와 문서 토큰은 upstream `vsdb`의 process-global base dir(`VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb`) 아래 keyspace에 `doc_id -> persisted snapshot JSON` catalog로 저장된다. 서버는 store 접근을 직렬화해 concurrent mutation을 막고, 기본 local ownership 모드에서는 same keyspace full scan으로 room을 eager hydrate한다. distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yakv`이면 snapshot과 문서 토큰이 `SNAPSHOT_YAKV_PATH` yakv 단일 B-Tree 파일의 `snapshot:<doc_id>` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yakv full scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=saberdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SABERDB_PATH` saberdb pretty JSON 파일의 `doc_id -> persisted snapshot JSON string` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=smolldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SMOLLDB_PATH` compressed SmollDB 파일의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 파일을 load해 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=kstone`이면 snapshot과 문서 토큰이 `SNAPSHOT_KSTONE_PATH` Kstone WAL/SSTable LSM 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=feoxdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_FEOXDB_PATH` FeOxDB 단일 파일의 `snapshot:<doc_id>:<timestamp>:<event_id>` immutable event key와 tombstone event에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 prefix range scan으로 최신 event를 선택해 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=jsondb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JSONDB_PATH` jsondb versioned pretty JSON 파일의 `snapshots.<doc_id>` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=kopperdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_KOPPERDB_PATH` 디렉터리 아래 kopperdb append-only 세그먼트의 `doc_id` key와 explicit `__catalog__` key에 저장된다. delete API가 없어 tombstone value로 삭제를 가리고, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하며 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=icefalldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_ICEFALLDB_PATH` 디렉터리의 `rsdb.log` append-only 로그에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 공개 delete API가 없어 tombstone value로 삭제를 가리고, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하며 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=eight`이면 snapshot과 문서 토큰이 `SNAPSHOT_EIGHT_PATH` 디렉터리 아래 eight filesystem storage의 `doc_<uuid_simple>` key tree에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 empty-prefix search catalog로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=epoch_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_EPOCH_DB_PATH` 디렉터리의 sled-backed multi-tree keyspace에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=ferrumdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_FERRUMDB_PATH` FerrumDB append-only log 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON value로 저장된다. save/delete 뒤 `FsyncPolicy::Always` 경계로 sync하고, 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=rumdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUMDB_PATH` 디렉터리의 append-only rumdb 로그 세트에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=koit`이면 snapshot과 문서 토큰이 `SNAPSHOT_KOIT_PATH` koit structured JSON 파일의 `snapshots.<doc_id>` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=lite_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_LITE_DB_PATH` LiteDb 디렉터리의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=log_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_LOG_KV_PATH` log_kv append-only 단일 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON string으로 저장된다. delete는 tombstone string으로 가리고, 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=lsm_storage_engine`이면 snapshot과 문서 토큰이 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. save/delete 뒤 `flush()`를 호출해 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=mindb`이면 snapshot과 문서 토큰이 `SNAPSHOT_MINDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. save/delete 뒤 `sync()`를 호출해 WAL durability 경계를 고정하고, reopen point index가 비어 있는 경우 adapter가 upstream `RecoveryManager`로 WAL을 재생해 catalog key와 snapshot payload를 읽는다. 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=mmdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_MMDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. sync write와 flush로 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=nanodb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NANODB_PATH` single JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry로 저장된다. save/delete 뒤 whole-file write로 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 whole-file object load로 room을 eager hydrate한다.
- `SNAPSHOT_STORE=jfs`이면 snapshot과 문서 토큰이 `SNAPSHOT_JFS_PATH` jfs single JSON 파일의 `doc_id -> persisted snapshot JSON string` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=json_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_JSON_STORE_PATH` append-only JSON line 파일의 `doc_id -> persisted snapshot` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file line replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=hmdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_HMDB_PATH` 디렉터리 아래 hmdb schema 로그 파일의 `doc_id -> persisted snapshot` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 append-only 로그 replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=nodb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NODB_PATH` nodb 단일 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 nodb key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=okofdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_OKOFDB_PATH` 디렉터리 아래 okofdb key-per-file storage의 `doc_<uuid_simple>` 파일 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same directory scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=simple_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_SIMPLE_DB_PATH` single-file simple_db store의 `doc_id -> base64(persisted snapshot JSON)` 라인 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=docdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_DOCDB_PATH` docdb JSON 파일의 `doc_id -> persisted snapshot` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=s3`이면 snapshot과 문서 토큰이 `SNAPSHOT_S3_ENDPOINT` / `SNAPSHOT_S3_BUCKET` / `SNAPSHOT_S3_PREFIX` 조합의 S3 object key `<prefix><doc_id>.json`에 저장된다. startup hydrate는 bucket listing 뒤 각 object를 읽어 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=managed`이면 snapshot과 문서 토큰이 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external snapshot service `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`를 통해 저장된다. 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SqliteSnapshotStore`는 row-level upsert로 기존 snapshot을 교체하며, 잘못된 timestamp나 손상된 row는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `HeedSnapshotStore`는 LMDB-backed named database upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `JammdbSnapshotStore`는 single-file B+ tree bucket upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `JanqlSnapshotStore`는 WAL/SSTable 디렉터리 keyspace upsert와 explicit `__catalog__` key를 함께 사용해 기존 snapshot을 교체하며, 손상된 snapshot payload나 missing catalog entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `CandystoreSnapshotStore`는 directory-backed append-only engine에 large payload를 `set_big/get_big`로 저장하고 `__catalog__` key를 별도로 유지하며, `flush`와 `checkpoint` 뒤 기존 snapshot을 교체한다. 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `FjallSnapshotStore`는 LSM-tree keyspace upsert 뒤 `PersistMode::SyncAll`로 journal을 동기화해 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `PersySnapshotStore`는 single-file copy-on-write segment update와 `doc_id -> record_id` replace index를 함께 사용해 기존 snapshot을 교체하며, 손상된 snapshot payload나 dangling index entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `ParityDbSnapshotStore`는 ordered BTree column upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `RedbSnapshotStore`는 key-value upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `MindbSnapshotStore`는 WAL/SSTable LSM keyspace upsert와 explicit `__catalog__` key를 함께 사용해 기존 snapshot을 교체하고 save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있는 경우 upstream `RecoveryManager` WAL replay fallback으로 catalog/snapshot을 읽으며, 손상된 snapshot payload나 missing catalog entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- 기본 `LocalRoomLocator`는 모든 문서를 현재 프로세스 소유로 해석한다.
- `StaticRoomLocator`는 `ROOM_OWNER_HINTS_PATH`의 문서별 owner 힌트를 읽고, 현재 `NODE_ID`와 다른 owner를 가진 문서에 대해 `409 conflict`와 owner 힌트를 반환한다.
- `FileRoomLocator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `SqliteRoomLocator`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `ROOM_COORDINATOR=noop`은 아무 side effect 없이 통과하고, `ROOM_COORDINATOR=logging`은 `NODE_ID`와 `doc_id` 기준 lifecycle log만 남긴다.
- `ROOM_COORDINATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`에 canonical lease state (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 atomic write로 남기고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release 방식으로 정리한다. `NODE_BASE_URL`이 주어지면 이 값도 canonical origin으로 정규화해 함께 기록한다.
- `ROOM_COORDINATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에 같은 canonical lease state를 upsert하고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 `node_id + lease_id + epoch` compare-and-delete로 정리한다. `NODE_BASE_URL`이 주어지면 canonical origin으로 정규화한 `base_url`도 함께 기록한다.
- `ROOM_LOCATOR=file`과 `ROOM_COORDINATOR=file`은 같은 `ROOM_COORDINATOR_STATE_DIR`를 공유해야 하며, 멀티 노드에서 쓰려면 각 노드가 같은 디렉터리를 읽고 쓸 수 있어야 한다.
- `ROOM_LOCATOR=sqlite`와 `ROOM_COORDINATOR=sqlite`는 같은 `ROOM_COORDINATOR_SQLITE_PATH`를 공유해야 하며, 실제 owner handoff를 원하면 shared snapshot store도 함께 맞춰야 한다.
- WebSocket 첫 세션 시작과 마지막 세션 종료 시점에 `RoomCoordinator` hook이 호출되도록 런타임 경계가 이미 연결돼 있다.
- 현재 file-backed lease state는 shared filesystem 위에서만 동작하는 best-effort 구현이다. crash 뒤에는 `expires_at` 경과 후에만 stale로 간주된다.
- `SqliteRoomCoordinator`/`SqliteRoomLocator`는 shared SQLite DB에서 transactional lease compare-and-swap을 수행한다. 실제 owner handoff는 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store와 함께 구성했을 때만 안전하게 활성화해야 한다.
- `ManagedRoomCoordinator`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에 `POST /v1/leases/:doc_id/acquire|renew|release`를 호출해 same canonical lease contract를 유지하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release를 요청한다. `ManagedRoomLocator`는 같은 service의 `GET /v1/leases/:doc_id`를 읽어 non-local owner를 판단한다.
- `ManagedSnapshotStore`는 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external snapshot service에 `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`를 호출해 document catalog와 full-state Yrs snapshot을 유지한다. optional `SNAPSHOT_MANAGED_AUTH_TOKEN`이 설정되면 모든 요청에 `Authorization: Bearer <token>` 헤더를 붙인다.
- `S3SnapshotStore`는 `SNAPSHOT_S3_ENDPOINT`, `SNAPSHOT_S3_BUCKET`, `SNAPSHOT_S3_PREFIX` 조합 아래의 S3-compatible object storage에 `<prefix><doc_id>.json` object를 저장하고, optional `SNAPSHOT_S3_SESSION_TOKEN`을 포함한 SigV4 요청으로 catalog/list/load/save/delete를 수행한다.
- managed lease service는 `Authorization: Bearer <ROOM_COORDINATION_MANAGED_AUTH_TOKEN>` 헤더를 선택적으로 받을 수 있고, conflict 시 현재 lease record를 `409` body로 반환해야 한다.

## Lease / Heartbeat Coordination Contract

- authoritative coordination store는 최소 `get`, `acquire`, `renew`, `release` 네 동작을 제공해야 한다. 현재 저장소에는 이 계약을 만족하는 SQLite 구현이 포함된다.
- owner record는 최소 `doc_id`, `node_id`, optional `base_url`, `lease_id`, `acquired_at`, `renewed_at`, `expires_at`, `epoch`를 저장해야 한다.
- `owner.base_url`을 노출하는 경우 현재 `StaticRoomLocator`와 같은 규칙을 따라 path/query 없는 origin-only absolute `http://` 또는 `https://` URL만 허용하고, 응답에는 canonical origin (`scheme://authority`)으로 실어야 한다.
- non-local owner conflict 응답은 항상 `x-collab-owner-node-id`를 포함하고, `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함해야 한다. redirect URL은 owner origin 뒤에 현재 요청의 path/query를 그대로 붙인 값이어야 한다.
- `lease_id`는 compare-and-swap 기준값이다. `renew`와 `release`는 현재 holder의 `lease_id`와 `node_id`가 모두 일치할 때만 성공해야 한다.
- `epoch`는 lease 재획득마다 증가하는 fencing token이다. snapshot write, redirect metadata, future async side effect는 이 값을 함께 기록해 stale owner가 늦게 도착한 작업을 덮어쓰지 못하게 해야 한다.
- `acquire`는 active lease가 없거나 `expires_at <= now`인 경우에만 새 owner를 기록해야 한다.
- `renew`는 첫 WebSocket 세션 시작 직후 background heartbeat loop에서 주기적으로 실행해야 하며, room이 active인 동안 `expires_at`을 앞으로 민다.
- `release`는 마지막 세션 종료 후 snapshot 저장이 성공한 뒤에만 호출해야 한다. snapshot 저장 실패 시 lease를 즉시 반환하지 말고 TTL 만료까지 기존 owner를 유지해야 한다.
- locator는 `expires_at`이 지나기 전까지는 non-local owner를 authoritative하게 취급하고, 만료 뒤에만 stale owner로 간주해야 한다. 단순 파일 mtime이나 로컬 clock drift만으로 조기 handoff를 결정하지 않는다.
- 권장 기본값은 `heartbeat_interval=10s`, `lease_ttl=30s`, `stale_after_missed_heartbeats=2`다. 즉, owner는 TTL의 절반보다 짧은 간격으로 renew를 시도하고, 다른 노드는 마지막 `expires_at`이 지난 뒤에만 ownership takeover를 시도한다.
- crash 복구 경로는 `owner crash -> renew 중단 -> expires_at 경과 -> 새 owner acquire -> snapshot restore -> room activate` 순서를 따른다. awareness는 재게시 허용 범위로 두고 내구성 복구 대상에는 포함하지 않는다.
- 현재 저장소의 `FileRoomCoordinator`/`FileRoomLocator`는 이 계약의 file-backed 준비 구현을 제공한다. canonical lease record, compare-and-release, background heartbeat renew, `expires_at` 기반 stale 판정은 로컬/shared filesystem 경계에서 검증할 수 있지만 여전히 best-effort rehearsal mode로만 사용해야 한다.
- 현재 저장소의 `SqliteRoomCoordinator`/`SqliteRoomLocator`는 같은 계약을 shared SQLite DB row에 매핑한 authoritative CAS 구현을 제공한다.
- 현재 저장소의 `ManagedRoomCoordinator`/`ManagedRoomLocator`는 external lease service를 쓰는 multi-host coordination backend를 제공하고, `ManagedSnapshotStore`는 같은 방식의 external durability backend를 제공한다. `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 `SNAPSHOT_STORE=sqlite`와 결합한 owner handoff rehearsal, `SNAPSHOT_STORE=managed` 자체의 저장/복구 경계, 그리고 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 `SNAPSHOT_STORE=managed`와 결합한 actual handoff rehearsal까지 모두 회귀 테스트로 검증됐다.

## Static Room Owner Hints

`ROOM_LOCATOR=static`일 때 `ROOM_OWNER_HINTS_PATH`는 아래 구조의 JSON 파일을 가리킨다.

```json
{
  "documents": {
    "00000000-0000-0000-0000-000000000000": {
      "node_id": "node-b",
      "base_url": "http://127.0.0.1:5001"
    }
  }
}
```

- `documents`에 없는 문서는 현재 노드 소유로 간주한다.
- `node_id`는 비어 있으면 안 된다.
- `node_id`와 `base_url`은 trim 후 저장된다.
- `base_url`은 선택값이며, 있으면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 한다.
- 유효한 `base_url`은 canonical origin (`scheme://authority`) 형태로 non-local owner conflict 응답의 `owner.base_url`에 실린다.

## Awareness Metadata Contract

Non-null awareness payloads are validated against `AwarenessState` on the WebSocket collaboration path. Malformed JSON and invalid field values are rejected before shared room awareness state is mutated.

WebSocket 연결 이후 클라이언트가 게시하는 Yrs awareness state는 아래 JSON 구조를 표준으로 사용한다.

```json
{
  "user": {
    "id": "user-7",
    "name": "Kim",
    "color": "#1f6feb"
  },
  "selection": {
    "anchor": 3,
    "head": 11
  },
  "client": {
    "id": "session-3",
    "kind": "editor"
  }
}
```

- `user.id`, `user.name`, `client.id`, `client.kind`는 비어 있으면 안 된다.
- `user.color`는 `#RRGGBB` 형식의 6자리 hex color를 사용한다.
- `selection`은 선택 사항이며, 커서/선택 범위를 공유하지 않을 때는 생략할 수 있다.
- 서버는 이 구조를 canonical contract로 문서화하고, 현재 단계에서는 awareness payload를 그대로 중계한다.
