# Setup

## Build

```bash
cargo check
```

## Run

```bash
cp .env.example .env
cargo run
```

기본 바인드 주소는 `127.0.0.1:4000`입니다.
기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`이므로 로컬 프런트엔드 개발 서버를 별도 포트에서 띄우는 흐름을 바로 재현할 수 있습니다.
기본 `API_TOKEN`은 `dev-admin-token`이며, 개발 환경에서는 이 토큰으로 문서 생성/목록 API를 호출합니다.
기본 `SNAPSHOT_STORE`는 `memory`이며, 프로세스 재시작 뒤에도 문서 snapshot을 유지하려면 `SNAPSHOT_STORE=file`과 `SNAPSHOT_DIR`, `SNAPSHOT_STORE=flash_kv`와 `SNAPSHOT_FLASH_KV_PATH`, `SNAPSHOT_STORE=highlandcows_isam`와 `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`, `SNAPSHOT_STORE=simple_db`와 `SNAPSHOT_SIMPLE_DB_PATH`, `SNAPSHOT_STORE=docdb`와 `SNAPSHOT_DOCDB_PATH`, `SNAPSHOT_STORE=eight`와 `SNAPSHOT_EIGHT_PATH`, `SNAPSHOT_STORE=epoch_db`와 `SNAPSHOT_EPOCH_DB_PATH`, `SNAPSHOT_STORE=rumdb`와 `SNAPSHOT_RUMDB_PATH`, `SNAPSHOT_STORE=sqlite`와 `SNAPSHOT_SQLITE_PATH`, `SNAPSHOT_STORE=heed`와 `SNAPSHOT_HEED_PATH`, `SNAPSHOT_STORE=hightower_kv`와 `SNAPSHOT_HIGHTOWER_KV_PATH`, `SNAPSHOT_STORE=hmdb`와 `SNAPSHOT_HMDB_PATH`, `SNAPSHOT_STORE=bitask`와 `SNAPSHOT_BITASK_PATH`, `SNAPSHOT_STORE=candystore`와 `SNAPSHOT_CANDYSTORE_PATH`, `SNAPSHOT_STORE=cuendillar`와 `SNAPSHOT_CUENDILLAR_PATH`, `SNAPSHOT_STORE=jammdb`와 `SNAPSHOT_JAMMDB_PATH`, `SNAPSHOT_STORE=fjall`와 `SNAPSHOT_FJALL_PATH`, `SNAPSHOT_STORE=persy`와 `SNAPSHOT_PERSY_PATH`, `SNAPSHOT_STORE=persistent_kv`와 `SNAPSHOT_PERSISTENT_KV_PATH`, `SNAPSHOT_STORE=native_db`와 `SNAPSHOT_NATIVE_DB_PATH`, `SNAPSHOT_STORE=nebari`와 `SNAPSHOT_NEBARI_PATH`, `SNAPSHOT_STORE=nikidb`와 `SNAPSHOT_NIKIDB_PATH`, `SNAPSHOT_STORE=nodb`와 `SNAPSHOT_NODB_PATH`, `SNAPSHOT_STORE=okofdb`와 `SNAPSHOT_OKOFDB_PATH`, `SNAPSHOT_STORE=parity_db`와 `SNAPSHOT_PARITY_DB_PATH`, `SNAPSHOT_STORE=pickledb`와 `SNAPSHOT_PICKLEDB_PATH`, `SNAPSHOT_STORE=microkv`와 `SNAPSHOT_MICROKV_PATH`, `SNAPSHOT_STORE=redb`와 `SNAPSHOT_REDB_PATH`, `SNAPSHOT_STORE=rskey`와 `SNAPSHOT_RSKEY_PATH`, `SNAPSHOT_STORE=readb`와 `SNAPSHOT_READB_PATH`, `SNAPSHOT_STORE=rustlite`와 `SNAPSHOT_RUSTLITE_PATH`, `SNAPSHOT_STORE=rusty_leveldb`와 `SNAPSHOT_RUSTY_LEVELDB_PATH`, `SNAPSHOT_STORE=canopydb`와 `SNAPSHOT_CANOPYDB_PATH`, `SNAPSHOT_STORE=caves`와 `SNAPSHOT_CAVES_PATH`, `SNAPSHOT_STORE=ckydb`와 `SNAPSHOT_CKYDB_PATH`, `SNAPSHOT_STORE=scdb`와 `SNAPSHOT_SCDB_PATH`, `SNAPSHOT_STORE=surrealkv`와 `SNAPSHOT_SURREALKV_PATH`, `SNAPSHOT_STORE=sled`와 `SNAPSHOT_SLED_PATH`, `SNAPSHOT_STORE=rustbreak`와 `SNAPSHOT_RUSTBREAK_PATH`, `SNAPSHOT_STORE=yedb`와 `SNAPSHOT_YEDB_PATH`, `SNAPSHOT_STORE=btree_store`와 `SNAPSHOT_BTREE_STORE_PATH`, `SNAPSHOT_STORE=siamesedb`와 `SNAPSHOT_SIAMESDB_PATH`, `SNAPSHOT_STORE=structsy`와 `SNAPSHOT_STRUCTSY_PATH`, `SNAPSHOT_STORE=abyssiniandb`와 `SNAPSHOT_ABYSSINIANDB_PATH`, `SNAPSHOT_STORE=aeternusdb`와 `SNAPSHOT_AETERNUSDB_PATH`, `SNAPSHOT_STORE=thunderdb`와 `SNAPSHOT_THUNDERDB_PATH`, `SNAPSHOT_STORE=tinybase`와 `SNAPSHOT_TINYBASE_PATH`, `SNAPSHOT_STORE=dblite`와 `SNAPSHOT_DBLITE_PATH`, `SNAPSHOT_STORE=dbless`와 `SNAPSHOT_DBLESS_PATH`, `SNAPSHOT_STORE=db_rs`와 `SNAPSHOT_DB_RS_PATH`, `SNAPSHOT_STORE=sanakirja`와 `SNAPSHOT_SANAKIRJA_PATH`, `SNAPSHOT_STORE=snaildb`와 `SNAPSHOT_SNAILDB_PATH`, `SNAPSHOT_STORE=tinykv`와 `SNAPSHOT_TINYKV_PATH`, `SNAPSHOT_STORE=saberdb`와 `SNAPSHOT_SABERDB_PATH`, `SNAPSHOT_STORE=jsondb`와 `SNAPSHOT_JSONDB_PATH`, `SNAPSHOT_STORE=koit`와 `SNAPSHOT_KOIT_PATH`, `SNAPSHOT_STORE=jfs`와 `SNAPSHOT_JFS_PATH`, `SNAPSHOT_STORE=json_store`와 `SNAPSHOT_JSON_STORE_PATH`, `SNAPSHOT_STORE=s3`와 `SNAPSHOT_S3_*`, 또는 `SNAPSHOT_STORE=managed`와 `SNAPSHOT_MANAGED_BASE_URL`을 함께 설정합니다.
기본 `ROOM_LOCATOR`는 `local`이며, `static`으로 바꾸면 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 설정해 문서별 owner 힌트를 읽습니다. `file`로 바꾸면 `ROOM_COORDINATOR_STATE_DIR` 아래의 active room state JSON을 읽고, `sqlite`로 바꾸면 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블을 읽어 현재 노드 비소유 문서를 거절합니다. `managed`로 바꾸면 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service `GET /v1/leases/:doc_id`를 읽어 현재 노드 비소유 문서를 거절합니다.
기본 `ROOM_COORDINATOR`는 `noop`이며, `logging`으로 바꾸면 room 활성/비활성 lifecycle을 `NODE_ID` 기준 tracing log로만 남깁니다. `file`로 바꾸면 `ROOM_COORDINATOR_STATE_DIR` 아래에 active room lease JSON을 남기고, `sqlite`로 바꾸면 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에 lease row를 남긴 뒤 `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS` / `ROOM_COORDINATOR_LEASE_TTL_SECS`에 맞춰 heartbeat를 갱신합니다. `managed`로 바꾸면 같은 heartbeat/TTL 정책을 유지한 채 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service `POST /v1/leases/:doc_id/acquire|renew|release`를 호출합니다.

## Test

```bash
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
```

- `preflight.sh commit`/`publish`는 stage/commit/push 차단을 점검한다.
- `verify.sh core`는 socket bind나 `.git` 쓰기 가능 여부와 무관한 검증만 실행한다.
- `verify.sh websocket`는 WebSocket/삭제 통합 테스트처럼 socket bind가 필요한 검증만 실행한다.

## Environment Variables

- `HOST`: 서버가 바인드할 호스트명 또는 IP
- `PORT`: 서버 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing subscriber 필터
- `API_TOKEN`: 문서 생성 및 목록 조회용 Bearer 토큰
- `SNAPSHOT_STORE`: `memory`, `file`, `flash_kv`, `highlandcows_isam`, `simple_db`, `docdb`, `eight`, `epoch_db`, `rumdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `bitask`, `candystore`, `cuendillar`, `jammdb`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nikidb`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `tinybase`, `dblite`, `dbless`, `db_rs`, `sanakirja`, `snaildb`, `tinykv`, `yakv`, `saberdb`, `jsondb`, `kv`, `koit`, `jfs`, `json_store`, `s3`, 또는 `managed`
- `SNAPSHOT_DIR`: file snapshot store 루트 디렉터리
- `SNAPSHOT_FLASH_KV_PATH`: flash-kv snapshot store 디렉터리 경로
- `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`: highlandcows-isam snapshot store path prefix. 실제 저장 파일은 `<path>.idb`, `<path>.idx`
- `SNAPSHOT_SIMPLE_DB_PATH`: simple_db snapshot store 단일 파일 경로
- `SNAPSHOT_DOCDB_PATH`: docdb snapshot store JSON 파일 경로
- `SNAPSHOT_EIGHT_PATH`: eight snapshot store 디렉터리 경로
- `SNAPSHOT_EPOCH_DB_PATH`: epoch-db snapshot store 디렉터리 경로
- `SNAPSHOT_RUMDB_PATH`: rumdb snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_SQLITE_PATH`: sqlite snapshot store DB 파일 경로
- `SNAPSHOT_HEED_PATH`: heed snapshot store DB 디렉터리 경로
- `SNAPSHOT_HIGHTOWER_KV_PATH`: hightower-kv snapshot store 데이터 디렉터리 경로
- `SNAPSHOT_HMDB_PATH`: hmdb snapshot store append-only 로그 디렉터리 경로
- `SNAPSHOT_BITASK_PATH`: bitask snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_CANDYSTORE_PATH`: candystore snapshot store 디렉터리 경로
- `SNAPSHOT_CUENDILLAR_PATH`: cuendillar snapshot store 루트 디렉터리 경로. 내부에 `wal/`, `sstable/` 디렉터리가 함께 생성된다
- `SNAPSHOT_JAMMDB_PATH`: jammdb snapshot store DB 파일 경로
- `SNAPSHOT_FJALL_PATH`: fjall snapshot store DB 디렉터리 경로
- `SNAPSHOT_PERSY_PATH`: persy snapshot store DB 파일 경로
- `SNAPSHOT_PERSISTENT_KV_PATH`: persistent-kv snapshot store 디렉터리 경로
- `SNAPSHOT_NATIVE_DB_PATH`: native_db snapshot store DB 파일 경로
- `SNAPSHOT_NEBARI_PATH`: nebari snapshot store 디렉터리 경로
- `SNAPSHOT_NIKIDB_PATH`: nikidb snapshot store DB 파일 경로
- `SNAPSHOT_NODB_PATH`: nodb snapshot store DB 파일 경로
- `SNAPSHOT_OKOFDB_PATH`: okofdb snapshot store 디렉터리 경로
- `SNAPSHOT_PARITY_DB_PATH`: parity-db snapshot store DB 디렉터리 경로
- `SNAPSHOT_PICKLEDB_PATH`: PickleDB snapshot store 파일 경로
- `SNAPSHOT_MICROKV_PATH`: MicroKV snapshot store base path. 실제 DB 파일은 `<path>.kv`
- `SNAPSHOT_REDB_PATH`: redb snapshot store DB 파일 경로
- `SNAPSHOT_RSKEY_PATH`: rskey snapshot store JSON hashmap 파일 경로
- `SNAPSHOT_READB_PATH`: readb snapshot store 디렉터리 경로
- `SNAPSHOT_RUSTLITE_PATH`: rustlite snapshot store 디렉터리 경로
- `SNAPSHOT_RUSTY_LEVELDB_PATH`: rusty-leveldb snapshot store 디렉터리 경로
- `SNAPSHOT_CANOPYDB_PATH`: canopydb snapshot store 디렉터리 경로
- `SNAPSHOT_CAVES_PATH`: caves snapshot store 디렉터리 경로
- `SNAPSHOT_CKYDB_PATH`: ckydb snapshot store 디렉터리 경로
- `SNAPSHOT_SCDB_PATH`: scdb snapshot store 디렉터리 경로
- `SNAPSHOT_SKV_PATH`: skv snapshot store base path. 실제 저장 파일은 `<path>.data`, `<path>.index`
- `SNAPSHOT_SURREALKV_PATH`: surrealkv snapshot store 단일 파일 경로
- `SNAPSHOT_SLED_PATH`: sled snapshot store DB 디렉터리 경로
- `SNAPSHOT_RUSTBREAK_PATH`: rustbreak snapshot store 단일 파일 경로
- `SNAPSHOT_YEDB_PATH`: yedb snapshot store DB 디렉터리 경로
- `SNAPSHOT_BTREE_STORE_PATH`: btree-store snapshot store 단일 파일 경로
- `SNAPSHOT_SIAMESDB_PATH`: siamesedb snapshot store DB 디렉터리 경로
- `SNAPSHOT_STRUCTSY_PATH`: structsy snapshot store 단일 파일 경로
- `SNAPSHOT_ABYSSINIANDB_PATH`: abyssiniandb snapshot store 단일 파일 경로
- `SNAPSHOT_AETERNUSDB_PATH`: aeternusdb snapshot store 디렉터리 경로
- `SNAPSHOT_THUNDERDB_PATH`: thunderdb snapshot store 단일 파일 경로
- `SNAPSHOT_TINYBASE_PATH`: tinybase snapshot store sled 디렉터리 경로
- `SNAPSHOT_DBLITE_PATH`: dblite snapshot store 단일 파일 경로
- `SNAPSHOT_DBLESS_PATH`: dbless snapshot store 단일 파일 경로
- `SNAPSHOT_DB_RS_PATH`: db-rs snapshot store append-only 로그 디렉터리 경로
- `SNAPSHOT_SANAKIRJA_PATH`: sanakirja snapshot store 단일 파일 경로
- `SNAPSHOT_SNAILDB_PATH`: snaildb snapshot store 디렉터리 경로
- `SNAPSHOT_TINYKV_PATH`: tinykv snapshot store JSON 파일 경로
- `SNAPSHOT_YAKV_PATH`: yakv snapshot store 단일 파일 경로
- `SNAPSHOT_SABERDB_PATH`: saberdb snapshot store JSON 파일 경로
- `SNAPSHOT_JSONDB_PATH`: jsondb snapshot store JSON 파일 경로
- `SNAPSHOT_KV_PATH`: kv snapshot store sled 디렉터리 경로
- `SNAPSHOT_KOIT_PATH`: koit snapshot store JSON 파일 경로
- `SNAPSHOT_JFS_PATH`: jfs snapshot store single JSON 파일 경로
- `SNAPSHOT_JSON_STORE_PATH`: json_store snapshot store append-only JSON line 파일 경로
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
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 루트 디렉터리이며, `ROOM_LOCATOR=file`이 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_SQLITE_PATH`: `ROOM_COORDINATOR=sqlite`일 때 active room lease row를 저장하는 SQLite DB 파일 경로이며, `ROOM_LOCATOR=sqlite`가 같은 DB를 읽는다
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease heartbeat 갱신 간격(초)
- `ROOM_COORDINATOR_LEASE_TTL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease 만료 TTL(초)
- `ROOM_COORDINATION_MANAGED_BASE_URL`: `ROOM_LOCATOR=managed` 또는 `ROOM_COORDINATOR=managed`일 때 external lease service base URL
- `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`: managed coordination service에 보낼 optional Bearer 토큰
- `ROOM_COORDINATION_MANAGED_TIMEOUT_SECS`: managed coordination service HTTP timeout(초)
- `NODE_ID`: 현재 collaboration node 식별자
- `NODE_BASE_URL`: 현재 collaboration node를 다른 노드에 안내할 때 사용할 canonical origin-only base URL. `ROOM_COORDINATOR=file|sqlite|managed` lease state와 `ROOM_LOCATOR=file|sqlite|managed` conflict 응답의 `owner.base_url`에 반영된다.
- non-local owner `409 conflict`가 발생하면 ingress/proxy가 바로 사용할 수 있도록 `x-collab-owner-node-id` 헤더가 항상 붙고, `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 붙는다.
- `ROOM_OWNER_HINTS_PATH`: `ROOM_LOCATOR=static`일 때 owner hints JSON 파일 경로

## Snapshot Store Selection Guide

운영 기본값을 고를 때는 "restart durability만 필요한가"와 "owner coordination까지 같은 plane에서 authoritative하게 해결해야 하는가"를 먼저 나눈다.

| 질문 | 선택 기준 |
| --- | --- |
| 실제 multi-node owner handoff가 필요한가 | `SNAPSHOT_STORE=sqlite`를 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite`와 함께 쓰거나, `SNAPSHOT_STORE=managed`를 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`와 함께 쓴다. embedded backend는 snapshot durability만 제공한다. |
| 단일 노드 재시작 복구만 필요하고 파일 단위 백업/교체가 중요하나 | `file`, `jammdb`, `persy`, `native_db`, `nikidb`, `nodb`, `redb`, `rskey`, `rustbreak`, `btree_store`, `structsy`, `abyssiniandb`, `surrealkv`, `thunderdb`, `dblite`, `dbless`, `sanakirja`, `tinykv`, `yakv`, `saberdb`, `jsondb`, `koit`, `jfs`, `json_store`, `simple_db`, `docdb` 중 단일 path 기반 store를 우선 검토한다. |
| 디렉터리 단위 엔진 백업/restore 절차가 더 자연스러운가 | `flash_kv`, `heed`, `hightower_kv`, `hmdb`, `bitask`, `candystore`, `epoch_db`, `rumdb`, `fjall`, `parity_db`, `readb`, `kv`, `rustlite`, `rusty_leveldb`, `canopydb`, `ckydb`, `scdb`, `sled`, `yedb`, `siamesedb`, `snaildb`처럼 디렉터리 기반 store를 쓴다. |
| 운영자가 payload를 직접 열어보며 수동 복구해야 하나 | `file`, `pickledb`, `microkv`, `docdb`, `json_store`가 가장 단순하다. 대신 binary engine보다 payload 크기와 catalog scan 비용을 더 보수적으로 본다. |
| pure-Rust/no-bindgen/no-native-conflict 제약을 현재 빌드 그래프에서 유지해야 하나 | 현재 landed baseline은 `flash_kv`, `simple_db`, `docdb`, `eight`, `epoch_db`, `rumdb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `dblite`, `dbless`, `db_rs`, `sanakirja`, `snaildb`, `tinykv`, `yakv`, `saberdb`, `jsondb`, `kv`, `koit`, `jfs`, `json_store`, `persistent_kv`, `nikidb`, `nodb`, `readb`, `rustlite`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `scdb`, `skv`, `surrealkv`, `rskey`, `hightower_kv`, `hmdb`, `bitask`, `candystore`, `nebari`다. 추가 후보를 검토할 때도 native `links` 충돌과 bindgen 필요 여부를 먼저 배제한다. |

backend별 운영 차이를 빠르게 확인하려면 아래 매트릭스를 기준으로 본다.

| Backend | 저장 단위 | 운영자 payload 가시성 | 손상/복구 주의점 | 제약 메모 |
| --- | --- | --- | --- | --- |
| `file` | 문서별 JSON 파일 | 가장 높음 | 파일 하나 손상 시 해당 문서만 직접 격리 가능 | baseline filesystem store |
| `heed` | 디렉터리 + LMDB data file | 낮음 | 엔진 파일 단위 백업이 필요하고 수동 entry 복구는 어렵다 | mmap 기반, pure-Rust baseline에는 포함하지 않음 |
| `hightower_kv` | 디렉터리 + log-structured segments/snapshots | 낮음 | `snapshot:<doc_id>` prefix scan을 쓰므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `hmdb` | 디렉터리 + append-only bincode log | 낮음 | schema 로그 replay로 catalog를 복구한다. tail truncation은 incomplete write로 흡수할 수 있지만, 중간 구간 손상이나 스키마 불일치는 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitask` | 디렉터리 + append-only active/immutable logs | 낮음 | explicit `__catalog__` key를 같은 log에 유지한다. startup에는 log replay로 keydir를 재구축하고, writer lock이 단일 프로세스만 허용되므로 shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `candystore` | 디렉터리 + append-only data/log/index files | 낮음 | large payload는 `set_big/get_big`로 저장하고 `__catalog__` key를 함께 유지한다. `flush`와 `checkpoint` 뒤 durable cursor를 전진시키므로 엔진 디렉터리 전체 백업이 필요하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jammdb` | 단일 파일 | 낮음 | bucket 내부 key는 분리되지만 payload는 binary라 수동 복구가 어렵다 | single-file backup에 유리 |
| `fjall` | 디렉터리 keyspace | 낮음 | LSM directory 전체를 함께 백업해야 한다 | directory-backed engine |
| `persy` | 단일 파일 + index | 낮음 | entry 단위 skip은 가능하지만 index 일관성 검증이 필요하다 | single-file engine |
| `persistent_kv` | 디렉터리 + WAL/snapshot set | 낮음 | snapshot 디렉터리 전체와 WAL/shard 파일을 함께 백업해야 하고, payload는 binary value라 수동 수정보다 재시작 복구 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `native_db` | 단일 파일 | 낮음 | primary-key catalog라 payload 직접 점검은 어렵다 | single-file engine |
| `nebari` | 디렉터리 + append-only tree store | 낮음 | `snapshots` tree range scan으로 catalog를 복구하므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nikidb` | 단일 파일 B+tree bucket store | 낮음 | explicit `__catalog__` key와 문서 payload가 같은 B+tree file에 함께 저장된다. 수동 payload inspection은 어렵지만 bucket upsert와 single-file backup 절차는 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nodb` | 단일 파일 DB | 중간 | map 전체를 dump/rename 경계로 다시 쓰고 reopen 시 전체 load에 의존하므로 file corruption 시 startup 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `parity_db` | 디렉터리 column store | 낮음 | ordered column 전체를 묶어 관리해야 한다 | directory-backed engine |
| `pickledb` | 단일 JSON 유사 DB 파일 | 높음 | 사람이 읽기 쉽지만 대용량 catalog에서는 scan 비용을 더 보수적으로 본다 | text-oriented store |
| `microkv` | 단일 `.kv` 파일 | 중간 | key-value 구조는 단순하지만 payload는 binary 직렬화라 완전 수동 복구엔 한계가 있다 | simple local KV |
| `redb` | 단일 파일 | 낮음 | tree 내부 payload는 직접 읽기 어렵지만 entry skip 전략과 잘 맞는다 | single-file engine |
| `rskey` | 단일 JSON hashmap 파일 | 높음 | store 전체를 한 파일로 다시 쓰므로 파일 손상이 startup 전체 복구 실패로 이어질 수 있다. 대신 `doc_id -> persisted snapshot JSON string` 구조라 수동 점검과 부분 복구는 쉽다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `readb` | 디렉터리 + append-only data/index | 낮음 | `__catalog__` key와 data/index 파일을 함께 백업해야 catalog 복구 경로가 유지된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kv` | 디렉터리 + sled tree keyspace | 낮음 | `snapshots` bucket의 `doc_id` key를 full scan해 catalog를 복구한다. payload는 JSON codec으로 직렬화되지만 engine 디렉터리 전체 백업이 기본 절차다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `epoch_db` | 디렉터리 + sled-backed multi-tree store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 JSON string으로 저장한다. payload inspection보다는 engine 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rumdb` | 디렉터리 + append-only Bitcask-style log set | 낮음 | `doc_id` key와 explicit `__catalog__` key를 append-only log에 저장하고 startup 전체 log replay로 keydir를 복구한다. directory 전체 백업/restore와 replay 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rustlite` | 디렉터리 + WAL/SSTable engine | 낮음 | `__catalog__` key와 engine 디렉터리를 함께 백업해야 catalog 복구 경로가 유지된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `canopydb` | 디렉터리 + transactional tree/WAL | 낮음 | `snapshots` tree iter scan으로 catalog를 복구하므로 engine 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `caves` | 디렉터리 + key-per-file | 높음 | key마다 별도 파일이라 payload 확인은 쉽지만, crate caveat상 `set/delete` 뒤 매번 sync를 보장하지 않으므로 crash-consistency는 운영 백업/복구 절차로 보완해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ckydb` | 디렉터리 + index/log/data files | 낮음 | `__catalog__` key와 ckydb 디렉터리 전체를 함께 백업해야 한다. payload는 base64 문자열이라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `scdb` | 디렉터리 + `dump.scdb` | 낮음 | `__catalog__` key와 scdb 디렉터리 전체를 함께 백업해야 한다. payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `skv` | data/index 파일 쌍 | 낮음 | `doc_id` key와 explicit `__catalog__` key를 `<path>.data`/`<path>.index` 파일 쌍에 저장하므로 두 파일을 함께 백업/restore해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `surrealkv` | 단일 파일 B+tree | 낮음 | full scan catalog는 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `sled` | 디렉터리 DB | 낮음 | 엔진 디렉터리 전체 백업과 restore가 기본 절차다 | directory-backed engine |
| `rustbreak` | 단일 파일 catalog | 중간 | catalog 전체 역직렬화 실패가 startup 실패로 이어질 수 있어 사전 백업 검증이 중요하다 | single-file but whole-file risk |
| `yedb` | 디렉터리 + per-key files | 중간 | key 파일이 나뉘어 있어 수동 탐색은 가능하지만 directory 전체 일관성을 같이 봐야 한다 | directory-backed text-friendly KV |
| `btree_store` | 단일 파일 | 낮음 | btree bucket은 binary지만 entry 단위 skip 전략과 잘 맞는다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `siamesedb` | 디렉터리 map store | 낮음 | map key는 분리되지만 engine iterator 특성 때문에 catalog key 보조 관리가 필요하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `structsy` | 단일 파일 record store | 중간 | record scan은 단순하지만 payload는 struct record라 임의 수정 대신 export/import 절차가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `abyssiniandb` | 단일 파일 key-value store | 낮음 | key/value는 단순하지만 payload와 catalog 모두 binary value라 수동 복구보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `aeternusdb` | 디렉터리 + WAL/SSTable LSM engine | 낮음 | `__catalog__` key와 엔진 디렉터리를 함께 백업해야 하고 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `thunderdb` | 단일 파일 transactional KV | 낮음 | bucket iter scan은 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dblite` | 단일 파일 append/reuse KV | 중간 | key index는 reopen 시 파일 전체 scan으로 재구성되고 file-level lock에 의존하므로, 단일 파일 백업은 단순하지만 partial file corruption 시 재구성 실패 가능성을 염두에 둬야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dbless` | 단일 파일 typed table store | 낮음 | redb-backed typed table이라 수동 payload inspection은 어렵지만 named table key scan은 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `db_rs` | 디렉터리 + append-only typed table log | 낮음 | `LookupTable<String, PersistedSnapshot>`가 append-only bincode log를 replay해 catalog를 재구성하므로 디렉터리 전체 백업이 필요하고, payload는 binary라 수동 수정 대신 회귀 테스트 기반 복구가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `sanakirja` | 단일 파일 copy-on-write B-tree | 낮음 | full scan catalog는 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `flash_kv` | 디렉터리 + append-only bitcask-style engine | 낮음 | `__catalog__` key와 active data file sync 경계를 함께 백업해야 하고, payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `snaildb` | 디렉터리 + WAL/SSTable LSM engine | 낮음 | `__catalog__` key와 엔진 디렉터리를 함께 백업해야 하고, payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `tinykv` | 단일 JSON 파일 store | 중간 | payload 가시성은 가장 높지만 whole-file rewrite와 전체 JSON 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `yakv` | 단일 B-Tree 파일 | 낮음 | `snapshot:<doc_id>` key를 직접 저장하고 full scan으로 catalog를 복구한다. payload는 binary value이고 파일 전체 무결성에 의존하므로 수동 수정 대신 whole-file backup/restore와 회귀 테스트가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `saberdb` | 단일 pretty JSON 파일 store | 중간 | atomic temp+rename은 단순하지만 catalog 전체를 pretty JSON으로 다시 쓰고 startup 시 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jsondb` | 단일 versioned pretty JSON 파일 store | 중간 | write guard drop마다 whole-file pretty JSON rewrite와 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `koit` | 단일 structured JSON 파일 store | 중간 | 전체 catalog를 메모리에 로드한 뒤 save마다 whole-file rewrite와 `sync_all`을 수행하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jfs` | 단일 JSON object store | 높음 | single-file catalog를 temp+rename으로 교체해 각 `doc_id` object를 저장한다. payload inspection은 쉽지만 whole-file parse에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `json_store` | 단일 append-only JSON line 파일 store | 높음 | key별 최신 line offset을 메모리 인덱스로 유지하므로 payload inspection은 쉽지만, compaction 없이는 append log가 계속 커지고 startup catalog rebuild는 전체 파일 replay에 의존한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `simple_db` | 단일 line-oriented text 파일 | 중간 | `doc_id:base64(payload)` 라인 전체를 다시 쓰므로 파일 단위 백업은 단순하지만 partial rewrite 시 최근 쓰기 일부가 유실될 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `docdb` | 단일 JSON 파일 store | 중간 | `doc_id -> persisted snapshot` map 전체를 temp+rename으로 다시 쓰므로 단일 파일 백업은 단순하지만 전체 역직렬화 실패가 startup 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |

- `btree_store`는 single-file embedded durability 기준선이다.
- `skv`는 paired data/index 파일 기반 embedded durability 최신 기준선이다.
- `siamesedb`는 directory-backed embedded durability 기준선이다.
- `structsy`는 single-file embedded durability 기준선이다.
- `abyssiniandb`는 single-file embedded durability 기준선이다.
- `aeternusdb`는 directory-backed LSM embedded durability 기준선이다.
- `hightower_kv`는 prefix-indexed directory-backed embedded durability 기준선이다.
- `hmdb`는 append-only bincode log directory-backed embedded durability 기준선이다.
- `surrealkv`는 single-file embedded durability 기준선이다.
- `thunderdb`는 single-file embedded durability 기준선이다.
- `dbless`는 redb-backed single-file typed table embedded durability 기준선이다.
- `sanakirja`는 single-file copy-on-write B-tree embedded durability 기준선이다.
- `flash_kv`는 append-only directory-backed embedded durability 기준선이다.
- `snaildb`는 WAL/SSTable directory-backed embedded durability 기준선이다.
- `tinykv`는 human-readable single-file embedded durability 기준선이다.
- `yakv`는 single-file B-Tree embedded durability 기준선이다.
- `saberdb`는 atomic temp+rename pretty JSON embedded durability 기준선이다.
- `jsondb`는 schema-versioned pretty JSON embedded durability 기준선이다.
- `koit`는 async whole-file structured JSON embedded durability 기준선이다.
- `jfs`는 single-file JSON object embedded durability 기준선이다.
- `persistent_kv`는 snapshot set + WAL directory embedded durability 기준선이다.
- `epoch_db`는 sled-backed multi-tree directory embedded durability 기준선이다.
- `nikidb`는 single-file B+tree bucket embedded durability 기준선이다.
- `nodb`는 single-file dump/rename embedded durability 기준선이다.
- `simple_db`는 line-oriented single-file embedded durability 기준선이다.
- `docdb`는 auto-dump single-file embedded durability 기준선이다.
- `rskey`는 single-file embedded durability 기준선이다.
- `readb`는 directory-backed embedded durability 기준선이다.
- `kv`는 sled tree bucket 기반 directory-backed embedded durability 기준선이다.
- `rustlite`는 directory-backed embedded durability 기준선이다.
- `canopydb`는 directory-backed embedded durability 기준선이다.
- `caves`는 key-per-file directory-backed embedded durability 기준선이다.
- `ckydb`는 directory-backed embedded durability 기준선이다.
- `scdb`는 directory-backed embedded durability 기준선이다.
- `rustbreak`는 catalog 파일 전체 역직렬화 실패가 startup 복구 실패로 이어질 수 있으므로 운영 기본값으로 둘 때 별도 백업/검증 절차가 필요하다.
- corrupt entry를 warning과 함께 건너뛰는 현재 catalog 정책을 적극 활용하려면 `flash_kv`, `hightower_kv`, `jammdb`, `persy`, `native_db`, `redb`, `btree_store`, `siamesedb`, `abyssiniandb`, `ckydb`, `scdb`, `skv`, `surrealkv`, `thunderdb`, `dblite`, `dbless`, `sanakirja`, `snaildb`, `simple_db` 쪽이 기본값 후보로 더 안전하다. `hmdb`는 tail truncation은 흡수할 수 있지만 로그 중간 손상 시 startup 전체 복구 실패로 이어질 수 있고, `nikidb`는 single-file bucket store라 backup 절차는 단순하지만 binary B+tree file 전체 무결성에 더 의존한다.
- `rskey`는 JSON hashmap 전체를 다시 쓰는 구조라 payload 가시성은 높지만 store 파일 하나 손상이 startup 전체 복구 실패로 이어질 수 있다.
- embedded backend를 고르더라도 실제 ownership authority는 `ROOM_COORDINATOR=file|static`에 두지 않는다. handoff가 필요하면 `sqlite` 또는 `managed` coordination과 조합한다.

## Static Room Locator File

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

- 힌트에 없는 문서는 현재 노드 소유로 간주한다.
- `node_id`와 `base_url`은 trim 후 저장된다.
- `base_url`은 선택값이며, 있으면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 한다.
- 유효한 `base_url`은 canonical origin (`scheme://authority`) 형태로 non-local owner `409` 응답의 owner metadata로 전달된다.

## File Room Locator State

- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽어 active owner lease를 판정한다.
- 이 모드는 `ROOM_COORDINATOR=file`이 남긴 state를 소비하는 전제이므로, 멀티 노드에서 사용하려면 각 노드가 같은 `ROOM_COORDINATOR_STATE_DIR`를 읽고 쓸 수 있어야 한다.
- `NODE_BASE_URL`이 설정된 노드가 `ROOM_COORDINATOR=file`을 사용하면 lease state에 canonical `base_url`도 기록되고, `ROOM_LOCATOR=file`의 non-local owner `409` 응답에도 `owner.base_url`이 함께 실린다.
- current file-backed state는 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 저장하고, stale owner 판단은 `expires_at` 기준으로만 수행한다.
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`는 0보다 커야 하고 `ROOM_COORDINATOR_LEASE_TTL_SECS`보다 작아야 한다.
- 이 구현은 shared filesystem 위에서만 best-effort로 동작한다.

## Sqlite Room Locator State

- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블을 읽어 active owner lease를 판정한다.
- 이 모드는 `ROOM_COORDINATOR=sqlite`가 남긴 lease row를 그대로 소비하므로, 실제 owner handoff를 원하면 각 노드가 같은 SQLite DB 파일을 lock-capable storage 위에서 공유해야 한다.
- `NODE_BASE_URL`이 설정된 노드가 `ROOM_COORDINATOR=sqlite`를 사용하면 lease row에 canonical `base_url`도 기록되고, `ROOM_LOCATOR=sqlite`의 non-local owner `409` 응답에도 `owner.base_url`이 함께 실린다.
- sqlite lease row도 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 저장하고, stale owner 판단은 `expires_at` 기준으로만 수행한다.
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`는 0보다 커야 하고 `ROOM_COORDINATOR_LEASE_TTL_SECS`보다 작아야 한다.
- 이 구현은 shared SQLite DB에서 transactional compare-and-swap을 제공하지만, 실제 handoff를 안전하게 쓰려면 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store를 함께 구성해야 한다.

## Managed Room Coordination Service

- `ROOM_LOCATOR=managed`와 `ROOM_COORDINATOR=managed`는 같은 `ROOM_COORDINATION_MANAGED_BASE_URL`을 공유해야 한다.
- base URL은 absolute `http://` 또는 `https://` URL이어야 하며 query string은 허용하지 않는다. path prefix는 허용되며, 실제 요청은 그 뒤에 `/v1/leases/:doc_id` 및 `/v1/leases/:doc_id/acquire|renew|release`가 붙는다.
- optional `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`이 설정되면 모든 managed lease 요청에 `Authorization: Bearer <token>` 헤더가 붙는다.
- lookup은 `GET /v1/leases/:doc_id`, acquire는 `POST /v1/leases/:doc_id/acquire`, renew는 `POST /v1/leases/:doc_id/renew`, release는 `POST /v1/leases/:doc_id/release`를 사용한다.
- `GET`과 성공한 `acquire`/`renew` 응답은 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 JSON으로 반환해야 한다.
- `acquire` 또는 `renew`/`release` conflict는 `409`와 현재 active lease record를 JSON body로 반환해야 한다.
- 이 구현은 coordination storage를 external service로 분리하지만, 실제 handoff를 안전하게 쓰려면 여전히 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store를 함께 구성해야 한다.

## Redb Snapshot Store

- `SNAPSHOT_STORE=heed`는 `SNAPSHOT_HEED_PATH` 단일 heed LMDB 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- heed `snapshots` database는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- `SNAPSHOT_STORE=hightower_kv`는 `SNAPSHOT_HIGHTOWER_KV_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 hightower-kv keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, `GET /api/documents` catalog는 same prefix scan으로 복원된다.
- `SNAPSHOT_STORE=jammdb`는 `SNAPSHOT_JAMMDB_PATH` 단일 jammdb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 jammdb `snapshots` bucket에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=fjall`는 `SNAPSHOT_FJALL_PATH` 단일 fjall DB 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 fjall `snapshots` keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=native_db`는 `SNAPSHOT_NATIVE_DB_PATH` 단일 native_db 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 native_db primary-key catalog에 `doc_id -> persisted snapshot JSON` payload로 저장된다.
- `SNAPSHOT_STORE=nebari`는 `SNAPSHOT_NEBARI_PATH` 디렉터리 아래 append-only tree store를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 nebari `snapshots` tree에 `doc_id -> persisted snapshot JSON` payload로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=nikidb`는 `SNAPSHOT_NIKIDB_PATH` 단일 nikidb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 nikidb `snapshots` bucket의 `doc_id -> persisted snapshot JSON` value와 explicit `__catalog__` key에 저장되고, document catalog는 same bucket 안의 catalog key로 복구된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=parity_db`는 `SNAPSHOT_PARITY_DB_PATH` 단일 parity-db 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 parity-db ordered `snapshots` column에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=redb`는 `SNAPSHOT_REDB_PATH` 단일 redb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 redb `snapshots` 테이블에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- `SNAPSHOT_STORE=rskey`는 `SNAPSHOT_RSKEY_PATH` 단일 rskey JSON hashmap 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rskey store에 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, `GET /api/documents` catalog는 same hashmap key scan으로 복원된다.
- `SNAPSHOT_STORE=readb`는 `SNAPSHOT_READB_PATH` readb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 readb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=rustlite`는 `SNAPSHOT_RUSTLITE_PATH` rustlite 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rustlite keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=rustcask`는 `SNAPSHOT_RUSTCASK_PATH` rustcask append-only Bitcask 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rustcask keyspace의 `doc_id` key에 저장되고, document catalog는 explicit `__catalog__` key와 startup log replay 결과로 복구된다.
- `SNAPSHOT_STORE=rumdb`는 `SNAPSHOT_RUMDB_PATH` rumdb append-only log 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rumdb keyspace의 `doc_id` key에 저장되고, document catalog는 explicit `__catalog__` key와 startup log replay 결과로 복구된다.
- `SNAPSHOT_STORE=rusty_leveldb`는 `SNAPSHOT_RUSTY_LEVELDB_PATH` rusty-leveldb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rusty-leveldb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, `GET /api/documents` catalog는 same keyspace full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- `SNAPSHOT_STORE=canopydb`는 `SNAPSHOT_CANOPYDB_PATH` canopydb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 canopydb `snapshots` tree에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- `SNAPSHOT_STORE=ckydb`는 `SNAPSHOT_CKYDB_PATH` ckydb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 ckydb key-value 엔트리에 `doc_id -> base64(persisted snapshot JSON)`로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=scdb`는 `SNAPSHOT_SCDB_PATH` scdb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 scdb key-value 엔트리에 `doc_id -> persisted snapshot JSON`으로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=skv`는 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data` / `<path>.index` 파일 쌍을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 skv key-value 엔트리에 `doc_id -> persisted snapshot JSON`으로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=surrealkv`는 `SNAPSHOT_SURREALKV_PATH` 단일 surrealkv B+tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 surrealkv keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=pickledb`는 `SNAPSHOT_PICKLEDB_PATH` 단일 PickleDB 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 PickleDB keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=microkv`는 `SNAPSHOT_MICROKV_PATH` base path를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 MicroKV 파일 `<path>.kv`에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=sled`는 `SNAPSHOT_SLED_PATH` 단일 sled DB 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 sled DB에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=rustbreak`는 `SNAPSHOT_RUSTBREAK_PATH` 단일 rustbreak 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rustbreak path database catalog에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- rustbreak catalog 파일 자체가 손상되면 startup 복구는 해당 파일을 역직렬화하지 못해 실패한다.
- `SNAPSHOT_STORE=yedb`는 `SNAPSHOT_YEDB_PATH` 단일 yedb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 yedb `snapshots/<doc_id>` key에 `persisted snapshot JSON`으로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=btree_store`는 `SNAPSHOT_BTREE_STORE_PATH` 단일 btree-store 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 btree-store `snapshots` bucket에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=siamesedb`는 `SNAPSHOT_SIAMESDB_PATH` 단일 siamesedb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 siamesedb `snapshots` map에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- `SNAPSHOT_STORE=structsy`는 `SNAPSHOT_STRUCTSY_PATH` 단일 structsy 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 structsy persistent record에 `doc_id`, title/timestamps/token, Yrs full-state update 필드로 저장된다.
- `SNAPSHOT_STORE=abyssiniandb`는 `SNAPSHOT_ABYSSINIANDB_PATH` 단일 abyssiniandb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 abyssiniandb `snapshots` map에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=aeternusdb`는 `SNAPSHOT_AETERNUSDB_PATH` 디렉터리 아래 WAL/SSTable 기반 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 aeternusdb keyspace에 `doc_id -> persisted snapshot JSON` binary value로 저장되고, 문서 catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=thunderdb`는 `SNAPSHOT_THUNDERDB_PATH` 단일 thunderdb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 thunderdb `snapshots` bucket에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- `SNAPSHOT_STORE=dblite`는 `SNAPSHOT_DBLITE_PATH` 단일 dblite 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 dblite string key-value 엔트리에 `doc_id -> persisted snapshot JSON` bytes로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=dbless`는 `SNAPSHOT_DBLESS_PATH` 단일 dbless 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 dbless typed table 엔트리에 `doc_id -> persisted snapshot`으로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=db_rs`는 `SNAPSHOT_DB_RS_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 db-rs `LookupTable<String, PersistedSnapshot>` 엔트리에 `doc_id -> persisted snapshot`으로 저장되고, document catalog는 append-only log replay 뒤 same table scan으로 복구된다.
- `SNAPSHOT_STORE=sanakirja`는 `SNAPSHOT_SANAKIRJA_PATH` 단일 sanakirja copy-on-write B-tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 sanakirja keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- `SNAPSHOT_STORE=snaildb`는 `SNAPSHOT_SNAILDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 snaildb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=tinykv`는 `SNAPSHOT_TINYKV_PATH` 단일 tinykv JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 tinykv keyspace에 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=yakv`는 `SNAPSHOT_YAKV_PATH` 단일 yakv B-Tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 yakv keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- `SNAPSHOT_STORE=saberdb`는 `SNAPSHOT_SABERDB_PATH` 단일 saberdb pretty JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 saberdb catalog에 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=jsondb`는 `SNAPSHOT_JSONDB_PATH` 단일 jsondb versioned pretty JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 jsondb catalog의 `snapshots.<doc_id>` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=kv`는 `SNAPSHOT_KV_PATH` sled 디렉터리와 `snapshots` bucket을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 kv catalog의 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 same bucket full scan으로 복구된다.
- `SNAPSHOT_STORE=eight`는 `SNAPSHOT_EIGHT_PATH` 디렉터리 아래 eight filesystem storage를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 eight keyspace의 `doc_<uuid_simple> -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 empty-prefix search 결과를 다시 load해 복구된다.
- `SNAPSHOT_STORE=koit`는 `SNAPSHOT_KOIT_PATH` 단일 koit structured JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 koit catalog의 `snapshots.<doc_id>` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=jfs`는 `SNAPSHOT_JFS_PATH` 단일 jfs JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 jfs catalog의 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=json_store`는 `SNAPSHOT_JSON_STORE_PATH` 단일 append-only JSON line 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 json_store catalog의 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 whole-file line replay와 key별 최신 offset 인덱스로 복구된다.
- `SNAPSHOT_STORE=hmdb`는 `SNAPSHOT_HMDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 hmdb schema 로그의 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 append-only 로그 replay로 복구된다.
- `SNAPSHOT_STORE=bitask`는 `SNAPSHOT_BITASK_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 bitask append-only log의 `doc_id` key와 explicit `__catalog__` key에 저장되고, document catalog는 startup log replay 뒤 재구축된 keydir를 따라 복구된다.
- `SNAPSHOT_STORE=candystore`는 `SNAPSHOT_CANDYSTORE_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 candystore keyspace의 `doc_id` key와 explicit `__catalog__` key에 저장되고, large payload는 `set_big/get_big`로 읽고 쓴 뒤 `flush`와 `checkpoint`로 durable cursor를 전진시킨다.
- `SNAPSHOT_STORE=caves`는 `SNAPSHOT_CAVES_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 caves key-per-file catalog의 `<doc_id>` 파일에 JSON bytes로 저장되고, document catalog는 directory scan으로 복구된다.
- `SNAPSHOT_STORE=persistent_kv`는 `SNAPSHOT_PERSISTENT_KV_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 persistent-kv keyspace에 `doc_id -> persisted snapshot JSON bytes` key-value로 저장되고, document catalog는 same key scan으로 복구된다.
- `SNAPSHOT_STORE=nodb`는 `SNAPSHOT_NODB_PATH` 단일 nodb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 nodb keyspace에 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=docdb`는 `SNAPSHOT_DOCDB_PATH` 단일 docdb JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 docdb keyspace에 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 same key scan으로 복구된다.
- `SNAPSHOT_STORE=shorterdb`는 `SNAPSHOT_SHORTERDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 shorterdb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.

## Managed Snapshot Service

- `SNAPSHOT_STORE=managed`는 `SNAPSHOT_MANAGED_BASE_URL`을 통해 external snapshot service를 사용한다.
- base URL은 absolute `http://` 또는 `https://` URL이어야 하며 query string은 허용하지 않는다. path prefix는 허용되며, 실제 요청은 그 뒤에 `/v1/snapshots`와 `/v1/snapshots/:doc_id`가 붙는다.
- optional `SNAPSHOT_MANAGED_AUTH_TOKEN`이 설정되면 모든 managed snapshot 요청에 `Authorization: Bearer <token>` 헤더가 붙는다.
- catalog lookup은 `GET /v1/snapshots`, load는 `GET /v1/snapshots/:doc_id`, save는 `PUT /v1/snapshots/:doc_id`, delete는 `DELETE /v1/snapshots/:doc_id`를 사용한다.
- `GET /v1/snapshots` 응답은 `{"documents":[...]}` shape로 document catalog를 반환해야 한다.
- `GET /v1/snapshots/:doc_id` 응답은 `{"document": {...}, "update": [...]}` shape로 full-state snapshot을 반환해야 한다. `document`에는 internal restore에 필요한 `id`, `title`, `created_at`, `updated_at`, `access_token`이 모두 포함돼야 한다.
- `PUT /v1/snapshots/:doc_id`는 같은 JSON payload를 받아 해당 문서 snapshot을 upsert해야 한다.
- `DELETE /v1/snapshots/:doc_id`는 문서 snapshot이 없어도 idempotent하게 성공해도 된다.
- 이 구현은 shared SQLite를 넘어서는 durability surface를 제공하며, `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`와 결합한 실제 owner handoff rehearsal도 회귀 테스트로 검증됐다.

## S3 Snapshot Store

- `SNAPSHOT_STORE=s3`는 `SNAPSHOT_S3_ENDPOINT`를 통해 S3-compatible object storage를 사용한다.
- endpoint는 absolute `http://` 또는 `https://` URL이어야 하며 query string은 허용하지 않는다.
- 필수 설정은 `SNAPSHOT_S3_ENDPOINT`, `SNAPSHOT_S3_REGION`, `SNAPSHOT_S3_BUCKET`, `SNAPSHOT_S3_ACCESS_KEY_ID`, `SNAPSHOT_S3_SECRET_ACCESS_KEY`다.
- optional `SNAPSHOT_S3_SESSION_TOKEN`이 설정되면 SigV4 요청에 session token도 함께 포함된다.
- `SNAPSHOT_S3_PREFIX`는 object key prefix이며, 최종 저장 경로는 `<prefix><doc_id>.json`이다.
- `SNAPSHOT_S3_PATH_STYLE=true`면 path-style addressing을 사용하고, `false`면 client 기본 auto addressing을 사용한다.
- `GET /api/documents` catalog는 bucket listing 뒤 matching object를 개별 load해 document metadata를 만든다.
- 이 구현은 vendor-specific object storage durability surface를 제공하며, startup hydrate와 `AppState::from_config` 재시작 복구 경계도 회귀 테스트로 검증됐다.

## Future Coordination Store Rollout Contract

- 실제 멀티 호스트 handoff를 shared SQLite DB 밖의 coordination plane으로 옮기려면 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 같은 외부 lease service에 연결한다.
- 그 backend는 최소 `GET /v1/leases/:doc_id`, `POST /v1/leases/:doc_id/acquire`, `POST /v1/leases/:doc_id/renew`, `POST /v1/leases/:doc_id/release` 네 API를 제공해야 한다.
- lease record는 `doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`를 저장해야 한다.
- `owner.base_url`을 응답에 노출하려면 현재 static hints와 같은 canonical origin 규칙을 따라야 한다.
- `renew`는 active room 동안 heartbeat loop로 반복되어야 하고, `release`는 마지막 세션 종료 뒤 snapshot 저장이 성공했을 때만 허용된다.
- stale owner 판단은 반드시 `expires_at` 기준으로만 해야 한다. 로컬 파일 timestamp나 프로세스 uptime만으로 handoff를 결정하지 않는다.
- 권장 기본값은 `heartbeat_interval=10s`, `lease_ttl=30s`, `max_missed_heartbeats_before_stale=2`다.
- 현재 저장소에는 filesystem rehearsal용 coordination surface, SQLite-backed authoritative coordination surface, vendor-specific embedded DB durability surface(`SNAPSHOT_STORE=heed|hightower_kv|hmdb|jammdb|fjall|persy|persistent_kv|native_db|nebari|nikidb|nodb|parity_db|pickledb|microkv|redb|rskey|readb|kv|eight|epoch_db|rumdb|rustlite|rusty_leveldb|canopydb|caves|ckydb|scdb|skv|surrealkv|sled|rustbreak|yedb|btree_store|siamesedb|structsy|abyssiniandb|aeternusdb|thunderdb|dblite|dbless|db_rs|sanakirja|snaildb|tinykv|yakv|saberdb|jsondb|koit|jfs|simple_db|docdb|shorterdb`), S3-compatible object storage durability surface, external lease service를 쓰는 managed coordination surface, 그리고 external snapshot service를 쓰는 managed durability surface가 함께 있다. shared snapshot durability 후보로 `SNAPSHOT_STORE=sqlite`를 쓸 수 있고, object storage durability 후보로 `SNAPSHOT_STORE=s3`를 쓸 수 있으며, 외부 service durability 후보로 `SNAPSHOT_STORE=managed`를 쓸 수 있다. 이를 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite` 또는 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`에 연결해 owner lease와 snapshot durability를 분리 구성할 수 있고, managed-managed actual handoff rehearsal까지 회귀 테스트로 검증됐다.

## Local Development Procedure

1. `.env.example`을 기준으로 로컬 환경값을 준비한다.
2. `cargo check`로 의존성과 컴파일 상태를 먼저 확인한다.
3. `cargo run`으로 서버를 올리고 `/api/health`를 확인한다.
4. `Authorization: Bearer <API_TOKEN>`으로 `POST /api/documents`를 호출해 문서를 만들고 응답의 `access_token`을 확보한다.
5. 문서 상세 조회, 삭제, WebSocket 연결에는 `Authorization: Bearer <access_token>`을 사용한다.
6. WebSocket 접속 시 `Origin` 헤더를 `FRONTEND_ORIGIN`과 맞춰 `/ws/:doc_id`에 접속한다.
7. 작업 시작 전에 `./scripts/verify.sh core`로 코드 경로를 먼저 검증하고, publish 전에는 `./scripts/preflight.sh publish`, WebSocket 검증 전에는 `./scripts/preflight.sh websocket`로 환경 차단을 확인한다.
8. 작업 마무리 전 `./scripts/verify.sh core`를 다시 실행하고, socket bind 가능한 러너에서는 `./scripts/verify.sh websocket`까지 실행한다.
9. `ROOM_LOCATOR=static`을 쓰는 경우에는 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 맞추고, non-local owner 문서에 대해 `409 conflict`, `owner` metadata, `x-collab-owner-node-id` 헤더가 반환되는지 확인한다. `ROOM_LOCATOR=file`을 쓰는 경우에는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json` state를 준비하고, `ROOM_LOCATOR=sqlite`를 쓰는 경우에는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 준비한다. `ROOM_LOCATOR=managed`를 쓰는 경우에는 managed lease service `GET /v1/leases/:doc_id`가 current lease record를 반환하도록 준비한 뒤 같은 응답이 `owner.node_id`, optional `owner.base_url`, optional `x-collab-redirect-location`/`Location` 기준으로 반환되는지 확인한다.
10. 재시작 복구를 검증하려면 `SNAPSHOT_STORE=file`, `SNAPSHOT_STORE=sqlite`, `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=hmdb`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=nebari`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=rskey`, `SNAPSHOT_STORE=readb`, `SNAPSHOT_STORE=kv`, `SNAPSHOT_STORE=eight`, `SNAPSHOT_STORE=epoch_db`, `SNAPSHOT_STORE=rustlite`, `SNAPSHOT_STORE=rusty_leveldb`, `SNAPSHOT_STORE=canopydb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=scdb`, `SNAPSHOT_STORE=skv`, `SNAPSHOT_STORE=surrealkv`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=db_rs`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=yakv`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=koit`, `SNAPSHOT_STORE=jfs`, `SNAPSHOT_STORE=simple_db`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=shorterdb`, `SNAPSHOT_STORE=s3`, 또는 `SNAPSHOT_STORE=managed`로 서버를 띄운 뒤 문서를 만든 다음 프로세스를 재시작해 같은 문서 ID가 hydrate되는지 확인한다. 단, `ROOM_LOCATOR != local` 또는 `ROOM_COORDINATOR=file|sqlite|managed` 같은 distributed ownership 모드에서는 startup eager hydrate 대신 ownership 확인 뒤 on-demand restore가 일어나므로, 실제 owner handoff 검증은 snapshot store와 authoritative coordination backend를 함께 맞춘 뒤 이전 owner 종료 후 새 owner의 detail/WS 진입이 최신 snapshot을 복구하는지 확인해야 한다.
