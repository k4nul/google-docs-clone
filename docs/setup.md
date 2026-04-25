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
기본 `SNAPSHOT_STORE`는 `memory`이며, 프로세스 재시작 뒤에도 문서 snapshot을 유지하려면 `SNAPSHOT_STORE=file`과 `SNAPSHOT_DIR`, `SNAPSHOT_STORE=agdb`와 `SNAPSHOT_AGDB_PATH`, `SNAPSHOT_STORE=amandine`와 `SNAPSHOT_AMANDINE_PATH`, `SNAPSHOT_STORE=apex_store`와 `SNAPSHOT_APEX_STORE_PATH`, `SNAPSHOT_STORE=armdb`와 `SNAPSHOT_ARMDB_PATH`, `SNAPSHOT_STORE=flash_kv`와 `SNAPSHOT_FLASH_KV_PATH`, `SNAPSHOT_STORE=ghaladb`와 `SNAPSHOT_GHALADB_PATH`, `SNAPSHOT_STORE=blockbucket`와 `SNAPSHOT_BLOCKBUCKET_PATH`, `SNAPSHOT_STORE=grebedb`와 `SNAPSHOT_GREBEDB_PATH`, `SNAPSHOT_STORE=grumpydb`와 `SNAPSHOT_GRUMPYDB_PATH`, `SNAPSHOT_STORE=graus_db`와 `SNAPSHOT_GRAUS_DB_PATH`, `SNAPSHOT_STORE=highlandcows_isam`와 `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`, `SNAPSHOT_STORE=simple_db`와 `SNAPSHOT_SIMPLE_DB_PATH`, `SNAPSHOT_STORE=docdb`와 `SNAPSHOT_DOCDB_PATH`, `SNAPSHOT_STORE=emdb`와 `SNAPSHOT_EMDB_PATH`, `SNAPSHOT_STORE=osmiumdb`와 `SNAPSHOT_OSMIUMDB_PATH`, `SNAPSHOT_STORE=eight`와 `SNAPSHOT_EIGHT_PATH`, `SNAPSHOT_STORE=epoch_db`와 `SNAPSHOT_EPOCH_DB_PATH`, `SNAPSHOT_STORE=etchdb`와 `SNAPSHOT_ETCHDB_PATH`, `SNAPSHOT_STORE=fastkv`와 `SNAPSHOT_FASTKV_PATH`, `SNAPSHOT_STORE=ferrumdb`와 `SNAPSHOT_FERRUMDB_PATH`, `SNAPSHOT_STORE=rumdb`와 `SNAPSHOT_RUMDB_PATH`, `SNAPSHOT_STORE=sqlite`와 `SNAPSHOT_SQLITE_PATH`, `SNAPSHOT_STORE=heed`와 `SNAPSHOT_HEED_PATH`, `SNAPSHOT_STORE=hightower_kv`와 `SNAPSHOT_HIGHTOWER_KV_PATH`, `SNAPSHOT_STORE=hmdb`와 `SNAPSHOT_HMDB_PATH`, `SNAPSHOT_STORE=hurrahdb`와 `SNAPSHOT_HURRAHDB_PATH`, `SNAPSHOT_STORE=fs_db`와 `SNAPSHOT_FS_DB_PATH`, `SNAPSHOT_STORE=sqjson`와 `SNAPSHOT_SQJSON_PATH`, `SNAPSHOT_STORE=icefalldb`와 `SNAPSHOT_ICEFALLDB_PATH`, `SNAPSHOT_STORE=bitask`와 `SNAPSHOT_BITASK_PATH`, `SNAPSHOT_STORE=bitkv_rs`와 `SNAPSHOT_BITKV_RS_PATH`, `SNAPSHOT_STORE=bitcask_engine`와 `SNAPSHOT_BITCASK_ENGINE_PATH`, `SNAPSHOT_STORE=blazeup`와 `SNAPSHOT_BLAZEUP_PATH`, `SNAPSHOT_STORE=candystore`와 `SNAPSHOT_CANDYSTORE_PATH`, `SNAPSHOT_STORE=celerix_store`와 `SNAPSHOT_CELERIX_STORE_PATH`, `SNAPSHOT_STORE=citadeldb`와 `SNAPSHOT_CITADELDB_PATH` 및 `SNAPSHOT_CITADELDB_PASSPHRASE`, `SNAPSHOT_STORE=cuendillar`와 `SNAPSHOT_CUENDILLAR_PATH`를 함께 설정합니다.
- `SNAPSHOT_STORE=data_pile`와 `SNAPSHOT_DATA_PILE_PATH`, `SNAPSHOT_STORE=jammdb`와 `SNAPSHOT_JAMMDB_PATH`, `SNAPSHOT_STORE=mace`와 `SNAPSHOT_MACE_PATH`, `SNAPSHOT_STORE=janql`와 `SNAPSHOT_JANQL_PATH`, `SNAPSHOT_STORE=jasondb`와 `SNAPSHOT_JASONDB_PATH`, `SNAPSHOT_STORE=jasonisnthappy`와 `SNAPSHOT_JASONISNTHAPPY_PATH`, `SNAPSHOT_STORE=fjall`와 `SNAPSHOT_FJALL_PATH`, `SNAPSHOT_STORE=persy`와 `SNAPSHOT_PERSY_PATH`, `SNAPSHOT_STORE=persistent_kv`와 `SNAPSHOT_PERSISTENT_KV_PATH`, `SNAPSHOT_STORE=native_db`와 `SNAPSHOT_NATIVE_DB_PATH`, `SNAPSHOT_STORE=nebari`와 `SNAPSHOT_NEBARI_PATH`, `SNAPSHOT_STORE=nikidb`와 `SNAPSHOT_NIKIDB_PATH`, `SNAPSHOT_STORE=nodb`와 `SNAPSHOT_NODB_PATH`, `SNAPSHOT_STORE=okofdb`와 `SNAPSHOT_OKOFDB_PATH`, `SNAPSHOT_STORE=parity_db`와 `SNAPSHOT_PARITY_DB_PATH`, `SNAPSHOT_STORE=pickledb`와 `SNAPSHOT_PICKLEDB_PATH`, `SNAPSHOT_STORE=rcask`와 `SNAPSHOT_RCASK_PATH`, `SNAPSHOT_STORE=microkv`와 `SNAPSHOT_MICROKV_PATH`, `SNAPSHOT_STORE=redb`와 `SNAPSHOT_REDB_PATH`, `SNAPSHOT_STORE=rskey`와 `SNAPSHOT_RSKEY_PATH`, `SNAPSHOT_STORE=readb`와 `SNAPSHOT_READB_PATH`, `SNAPSHOT_STORE=rustlite`와 `SNAPSHOT_RUSTLITE_PATH`, `SNAPSHOT_STORE=rusty_leveldb`와 `SNAPSHOT_RUSTY_LEVELDB_PATH`, `SNAPSHOT_STORE=canopydb`와 `SNAPSHOT_CANOPYDB_PATH`, `SNAPSHOT_STORE=caves`와 `SNAPSHOT_CAVES_PATH`, `SNAPSHOT_STORE=ckydb`와 `SNAPSHOT_CKYDB_PATH`, `SNAPSHOT_STORE=scdb`와 `SNAPSHOT_SCDB_PATH`, `SNAPSHOT_STORE=surrealkv`와 `SNAPSHOT_SURREALKV_PATH`, `SNAPSHOT_STORE=sled`와 `SNAPSHOT_SLED_PATH`, `SNAPSHOT_STORE=rustbreak`와 `SNAPSHOT_RUSTBREAK_PATH`, `SNAPSHOT_STORE=yedb`와 `SNAPSHOT_YEDB_PATH`, `SNAPSHOT_STORE=btree_store`와 `SNAPSHOT_BTREE_STORE_PATH`, `SNAPSHOT_STORE=cacache`와 `SNAPSHOT_CACACHE_PATH`, `SNAPSHOT_STORE=siamesedb`와 `SNAPSHOT_SIAMESDB_PATH`, `SNAPSHOT_STORE=structsy`와 `SNAPSHOT_STRUCTSY_PATH`, `SNAPSHOT_STORE=abyssiniandb`와 `SNAPSHOT_ABYSSINIANDB_PATH`, `SNAPSHOT_STORE=aeternusdb`와 `SNAPSHOT_AETERNUSDB_PATH`, `SNAPSHOT_STORE=thunderdb`와 `SNAPSHOT_THUNDERDB_PATH`, `SNAPSHOT_STORE=thetadb`와 `SNAPSHOT_THETADB_PATH`, `SNAPSHOT_STORE=tinybase`와 `SNAPSHOT_TINYBASE_PATH`, `SNAPSHOT_STORE=tinydb`와 `SNAPSHOT_TINYDB_PATH`, `SNAPSHOT_STORE=dblite`와 `SNAPSHOT_DBLITE_PATH`, `SNAPSHOT_STORE=dbless`와 `SNAPSHOT_DBLESS_PATH`, `SNAPSHOT_STORE=db_rs`와 `SNAPSHOT_DB_RS_PATH`, `SNAPSHOT_STORE=dharmadb`와 `SNAPSHOT_DHARMADB_PATH`, `SNAPSHOT_STORE=sanakirja`와 `SNAPSHOT_SANAKIRJA_PATH`, `SNAPSHOT_STORE=saturn`과 `SNAPSHOT_SATURN_PATH`, `SNAPSHOT_STORE=snaildb`와 `SNAPSHOT_SNAILDB_PATH`, `SNAPSHOT_STORE=tinykv`와 `SNAPSHOT_TINYKV_PATH`, `SNAPSHOT_STORE=vsdb`와 `SNAPSHOT_VSDB_PATH`, `SNAPSHOT_STORE=yakv`와 `SNAPSHOT_YAKV_PATH`, `SNAPSHOT_STORE=yakvdb`와 `SNAPSHOT_YAKVDB_PATH`, `SNAPSHOT_STORE=saberdb`와 `SNAPSHOT_SABERDB_PATH`, `SNAPSHOT_STORE=smolldb`와 `SNAPSHOT_SMOLLDB_PATH`, `SNAPSHOT_STORE=kstone`와 `SNAPSHOT_KSTONE_PATH`, `SNAPSHOT_STORE=roughdb`와 `SNAPSHOT_ROUGHDB_PATH`, `SNAPSHOT_STORE=raindb`와 `SNAPSHOT_RAINDB_PATH`, `SNAPSHOT_STORE=infusedb`와 `SNAPSHOT_INFUSEDB_PATH`, `SNAPSHOT_STORE=kafi`와 `SNAPSHOT_KAFI_PATH`, `SNAPSHOT_STORE=tinkv`와 `SNAPSHOT_TINKV_PATH`, `SNAPSHOT_STORE=ledger_kv`와 `SNAPSHOT_LEDGER_KV_PATH`, `SNAPSHOT_STORE=jsondb`와 `SNAPSHOT_JSONDB_PATH`, `SNAPSHOT_STORE=joydb`와 `SNAPSHOT_JOYDB_PATH`, `SNAPSHOT_STORE=png_db`와 `SNAPSHOT_PNG_DB_PATH`, `SNAPSHOT_STORE=kopperdb`와 `SNAPSHOT_KOPPERDB_PATH`, `SNAPSHOT_STORE=kv`와 `SNAPSHOT_KV_PATH`, `SNAPSHOT_STORE=koit`와 `SNAPSHOT_KOIT_PATH`, `SNAPSHOT_STORE=lite_db`와 `SNAPSHOT_LITE_DB_PATH`, `SNAPSHOT_STORE=lmdb_rs_core`와 `SNAPSHOT_LMDB_RS_CORE_PATH`, `SNAPSHOT_STORE=log_kv`와 `SNAPSHOT_LOG_KV_PATH`, `SNAPSHOT_STORE=append_log`와 `SNAPSHOT_APPEND_LOG_PATH`, `SNAPSHOT_STORE=loro_kv`와 `SNAPSHOT_LORO_KV_PATH`, `SNAPSHOT_STORE=luckdb`와 `SNAPSHOT_LUCKDB_PATH`, `SNAPSHOT_STORE=ipjdb`와 `SNAPSHOT_IPJDB_PATH`, `SNAPSHOT_STORE=kagi`와 `SNAPSHOT_KAGI_PATH`, `SNAPSHOT_STORE=deeb`와 `SNAPSHOT_DEEB_PATH`, `SNAPSHOT_STORE=lsm_engine`와 `SNAPSHOT_LSM_ENGINE_PATH`, `SNAPSHOT_STORE=lsm_storage_engine`와 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH`, `SNAPSHOT_STORE=lsmdb`와 `SNAPSHOT_LSMDB_PATH`, `SNAPSHOT_STORE=lsm_tree`와 `SNAPSHOT_LSM_TREE_PATH`, `SNAPSHOT_STORE=mindb`와 `SNAPSHOT_MINDB_PATH`, `SNAPSHOT_STORE=mmdb`와 `SNAPSHOT_MMDB_PATH`, `SNAPSHOT_STORE=nanodb`와 `SNAPSHOT_NANODB_PATH`, `SNAPSHOT_STORE=jfs`와 `SNAPSHOT_JFS_PATH`, `SNAPSHOT_STORE=json_store`와 `SNAPSHOT_JSON_STORE_PATH`, `SNAPSHOT_STORE=json_db_rs`와 `SNAPSHOT_JSON_DB_RS_PATH`, `SNAPSHOT_STORE=cdb64`와 `SNAPSHOT_CDB64_PATH`, `SNAPSHOT_STORE=json_mutex_db`와 `SNAPSHOT_JSON_MUTEX_DB_PATH`, `SNAPSHOT_STORE=toiletdb`와 `SNAPSHOT_TOILETDB_PATH`, `SNAPSHOT_STORE=s3`와 `SNAPSHOT_S3_*`, 또는 `SNAPSHOT_STORE=managed`와 `SNAPSHOT_MANAGED_BASE_URL`을 함께 설정합니다.
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
- `SNAPSHOT_STORE`: `memory`, `file`, `agdb`, `amandine`, `apex_store`, `armdb`, `assystem`, `colon_db`, `flash_kv`, `ghaladb`, `blockbucket`, `grebedb`, `grumpydb`, `graus_db`, `highlandcows_isam`, `simple_db`, `docdb`, `emdb`, `osmiumdb`, `eight`, `epoch_db`, `etchdb`, `fastkv`, `ferrumdb`, `rumdb`, `rubin`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `hurrahdb`, `fs_db`, `sqjson`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `citadeldb`, `cuendillar`, `data_pile`, `jammdb`, `mace`, `janql`, `jasondb`, `jasonisnthappy`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nikidb`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `rcask`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `crystal`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `cacache`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `dir_cache`, `sanakirja`, `saturn`, `snaildb`, `tinykv`, `vsdb`, `yakv`, `yakvdb`, `saberdb`, `smolldb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `jsondb`, `joydb`, `png_db`, `kopperdb`, `kv`, `koit`, `lite_db`, `lmdb_rs_core`, `log_kv`, `append_log`, `mhdb`, `marble`, `loro_kv`, `luckdb`, `deeb`, `lsm_engine`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `mu_db`, `nanodb`, `jfs`, `json_store`, `json_db_rs`, `cdb64`, `json_mutex_db`, `toiletdb`, `feoxdb`, `s3`, 또는 `managed`
- `SNAPSHOT_STORE=append_kv`: append_kv append-only 단일 파일 store도 지원한다.
- `SNAPSHOT_DIR`: file snapshot store 루트 디렉터리
- `SNAPSHOT_AGDB_PATH`: agdb snapshot store 단일 파일 경로
- `SNAPSHOT_AMANDINE_PATH`: Amandine snapshot store 디렉터리 경로
- `SNAPSHOT_APEX_STORE_PATH`: ApexStore snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_ARMDB_PATH`: armdb snapshot store 디렉터리 경로
- `SNAPSHOT_ASSYSTEM_PATH`: assystem snapshot store 단일 파일 경로
- `SNAPSHOT_COLON_DB_PATH`: colon_db snapshot store 단일 파일 경로
- `SNAPSHOT_FLASH_KV_PATH`: flash-kv snapshot store 디렉터리 경로
- `SNAPSHOT_GHALADB_PATH`: `SNAPSHOT_STORE=ghaladb`일 때 snapshot GhalaDB LSM value-log 디렉터리 경로
- `SNAPSHOT_BLOCKBUCKET_PATH`: blockbucket snapshot store 단일 파일 경로
- `SNAPSHOT_GREBEDB_PATH`: grebedb snapshot store 디렉터리 경로
- `SNAPSHOT_GRUMPYDB_PATH`: grumpydb snapshot store 디렉터리 경로
- `SNAPSHOT_GRAUS_DB_PATH`: GrausDb snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`: highlandcows-isam snapshot store path prefix. 실제 저장 파일은 `<path>.idb`, `<path>.idx`
- `SNAPSHOT_SIMPLE_DB_PATH`: simple_db snapshot store 단일 파일 경로
- `SNAPSHOT_DOCDB_PATH`: docdb snapshot store JSON 파일 경로
- `SNAPSHOT_EMDB_PATH`: emdb snapshot store DB 파일 경로. adapter는 `EmdbBuilder::prefer_v4(true)`와 explicit flush를 사용해 v0.7 engine 경계를 고정한다
- `SNAPSHOT_OSMIUMDB_PATH`: OsmiumDB snapshot store 디렉터리 경로. adapter는 save/delete마다 `flush()` 뒤 `checkpoint()`를 호출해 WAL replay와 map snapshot reopen 경계를 함께 고정한다
- `SNAPSHOT_EIGHT_PATH`: eight snapshot store 디렉터리 경로
- `SNAPSHOT_EPOCH_DB_PATH`: epoch-db snapshot store 디렉터리 경로
- `SNAPSHOT_ETCHDB_PATH`: EtchDB WAL-backed snapshot store 디렉터리 경로
- `SNAPSHOT_FASTKV_PATH`: FastKV compressed binary dump snapshot store 파일 경로
- `SNAPSHOT_RUMDB_PATH`: rumdb snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_SQLITE_PATH`: sqlite snapshot store DB 파일 경로
- `SNAPSHOT_HEED_PATH`: heed snapshot store DB 디렉터리 경로
- `SNAPSHOT_HIGHTOWER_KV_PATH`: hightower-kv snapshot store 데이터 디렉터리 경로
- `SNAPSHOT_HMDB_PATH`: hmdb snapshot store append-only 로그 디렉터리 경로
- `SNAPSHOT_HURRAHDB_PATH`: hurrahdb snapshot store append-only 파일 경로
- `SNAPSHOT_FS_DB_PATH`: fs-db snapshot store key-per-file 디렉터리 경로
- `SNAPSHOT_SQJSON_PATH`: `SNAPSHOT_STORE=sqjson`일 때 snapshot sqjson single-file JSON DB 경로
- `SNAPSHOT_BITASK_PATH`: bitask snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_BITKV_RS_PATH`: bitkv-rs snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_BITCASK_ENGINE_PATH`: bitcask-engine-rs snapshot store append-only log 디렉터리 경로
- `SNAPSHOT_BLAZEUP_PATH`: blazeup snapshot store의 kv/sled 디렉터리 경로
- `SNAPSHOT_CANDYSTORE_PATH`: candystore snapshot store 디렉터리 경로
- `SNAPSHOT_CELERIX_STORE_PATH`: celerix_store snapshot store 디렉터리 경로
- `SNAPSHOT_CITADELDB_PATH`: citadeldb encrypted snapshot DB 파일 경로
- `SNAPSHOT_CITADELDB_PASSPHRASE`: citadeldb key file passphrase
- `SNAPSHOT_CUENDILLAR_PATH`: cuendillar snapshot store 루트 디렉터리 경로. 내부에 `wal/`, `sstable/` 디렉터리가 함께 생성된다
- `SNAPSHOT_DATA_PILE_PATH`: `SNAPSHOT_STORE=data_pile`일 때 snapshot data-pile append-only 디렉터리 경로
- `SNAPSHOT_DATASTACK_PATH`: DataStack snapshot store redb 파일 경로
- `SNAPSHOT_JAMMDB_PATH`: jammdb snapshot store DB 파일 경로
- `SNAPSHOT_MACE_PATH`: Mace snapshot store 디렉터리 경로
- `SNAPSHOT_JANQL_PATH`: janql snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_JASONDB_PATH`: jasondb snapshot store append-only 단일 파일 경로
- `SNAPSHOT_JASONISNTHAPPY_PATH`: jasonisnthappy snapshot store 단일 DB 파일 경로
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
- `SNAPSHOT_RCASK_PATH`: RCask snapshot store 세그먼트 디렉터리 경로
- `SNAPSHOT_MICROKV_PATH`: MicroKV snapshot store base path. 실제 DB 파일은 `<path>.kv`
- `SNAPSHOT_REDB_PATH`: redb snapshot store DB 파일 경로
- `SNAPSHOT_RSKEY_PATH`: rskey snapshot store JSON hashmap 파일 경로
- `SNAPSHOT_READB_PATH`: readb snapshot store 디렉터리 경로
- `SNAPSHOT_RUSTLITE_PATH`: rustlite snapshot store 디렉터리 경로
- `SNAPSHOT_RUSTY_LEVELDB_PATH`: rusty-leveldb snapshot store 디렉터리 경로
- `SNAPSHOT_CANOPYDB_PATH`: canopydb snapshot store 디렉터리 경로
- `SNAPSHOT_CAVES_PATH`: caves snapshot store 디렉터리 경로
- `SNAPSHOT_CKYDB_PATH`: ckydb snapshot store 디렉터리 경로
- `SNAPSHOT_CREPEDB_PATH`: CrepeDB redb snapshot store 단일 파일 경로
- `SNAPSHOT_CRYSTAL_PATH`: crystal snapshot store key-per-file 디렉터리 경로
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
- `SNAPSHOT_THETADB_PATH`: thetadb snapshot store 단일 파일 경로
- `SNAPSHOT_VSDB_PATH`: vsdb snapshot store handle metadata 디렉터리 경로 (`store.meta.json`). 실제 keyspace는 upstream `vsdb`의 process-global base dir(`VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb`)을 따른다
- `SNAPSHOT_TINYBASE_PATH`: tinybase snapshot store sled 디렉터리 경로
- `SNAPSHOT_TINYDB_PATH`: tinydb snapshot store bincode 단일 파일 경로
- `SNAPSHOT_DBLITE_PATH`: dblite snapshot store 단일 파일 경로
- `SNAPSHOT_DBLESS_PATH`: dbless snapshot store 단일 파일 경로
- `SNAPSHOT_DB_RS_PATH`: db-rs snapshot store append-only 로그 디렉터리 경로
- `SNAPSHOT_DHARMADB_PATH`: dharmadb snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_SANAKIRJA_PATH`: sanakirja snapshot store 단일 파일 경로
- `SNAPSHOT_SNAILDB_PATH`: snaildb snapshot store 디렉터리 경로
- `SNAPSHOT_TINYKV_PATH`: tinykv snapshot store JSON 파일 경로
- `SNAPSHOT_YAKV_PATH`: yakv snapshot store 단일 파일 경로
- `SNAPSHOT_YAKVDB_PATH`: yakvdb snapshot store 단일 파일 경로
- `SNAPSHOT_SABERDB_PATH`: saberdb snapshot store JSON 파일 경로
- `SNAPSHOT_SMOLLDB_PATH`: smolldb snapshot store compressed 단일 파일 경로
- `SNAPSHOT_KSTONE_PATH`: kstone snapshot store WAL/SSTable LSM 디렉터리 경로
- `SNAPSHOT_ROUGHDB_PATH`: roughdb snapshot store LevelDB-compatible WAL/SSTable 디렉터리 경로
- `SNAPSHOT_RAINDB_PATH`: raindb snapshot store LevelDB-style WAL/SSTable 디렉터리 경로
- `SNAPSHOT_INFUSEDB_PATH`: infusedb snapshot store 단일 파일 경로
- `SNAPSHOT_KAFI_PATH`: kafi snapshot store 단일 파일 경로
- `SNAPSHOT_TINKV_PATH`: tinkv snapshot store append-only data 디렉터리 경로
- `SNAPSHOT_LEDGER_KV_PATH`: ledger-kv snapshot store append-only ledger 디렉터리 경로
- `SNAPSHOT_FEOXDB_PATH`: feoxdb snapshot store 단일 파일 경로
- `SNAPSHOT_JSONDB_PATH`: jsondb snapshot store JSON 파일 경로
- `SNAPSHOT_JOYDB_PATH`: joydb snapshot store JSON 파일 경로
- `SNAPSHOT_PNG_DB_PATH`: png_db snapshot store PNG 파일 경로
- `SNAPSHOT_KOPPERDB_PATH`: kopperdb snapshot store 세그먼트 디렉터리 경로
- `SNAPSHOT_ICEFALLDB_PATH`: icefalldb snapshot store 로그 디렉터리 경로
- `SNAPSHOT_KV_PATH`: kv snapshot store sled 디렉터리 경로
- `SNAPSHOT_KOIT_PATH`: koit snapshot store JSON 파일 경로
- `SNAPSHOT_LITE_DB_PATH`: lite_db snapshot store LiteDb 디렉터리 경로
- `SNAPSHOT_LOG_KV_PATH`: log_kv snapshot store append-only 단일 파일 경로
- `SNAPSHOT_APPEND_KV_PATH`: append_kv snapshot store append-only 단일 파일 경로
- `SNAPSHOT_APPEND_LOG_PATH`: append-log snapshot store append-only 단일 파일 경로
- `SNAPSHOT_MHDB_PATH`: mhdb snapshot store DB path prefix. 실제 저장 파일은 `<path>.pag`, `<path>.dir`
- `SNAPSHOT_LORO_KV_PATH`: loro-kv-store snapshot store binary SSTable 파일 경로
- `SNAPSHOT_LUCKDB_PATH`: luckdb snapshot store JSON document 파일 경로
- `SNAPSHOT_IPJDB_PATH`: ipjdb snapshot store collection 디렉터리 경로
- `SNAPSHOT_DEEB_PATH`: Deeb snapshot store JSON database 파일 경로
- `SNAPSHOT_RUBIN_PATH`: rubin snapshot store JSON 파일 경로
- `SNAPSHOT_LSM_ENGINE_PATH`: lsm_engine snapshot store WAL 파일 경로
- `SNAPSHOT_LSM_STORAGE_ENGINE_PATH`: lsm_storage_engine snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_LSMDB_PATH`: lsmdb snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_FASTKV_PATH`: FastKV snapshot store compressed binary dump 파일 경로
- `SNAPSHOT_FERRUMDB_PATH`: ferrumdb snapshot store append-only log 파일 경로
- `SNAPSHOT_MINDB_PATH`: Mindb snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_MMDB_PATH`: MMDB snapshot store WAL/SSTable 디렉터리 경로
- `SNAPSHOT_MU_DB_PATH`: muDB snapshot store data 파일 경로. 같은 디렉터리의 `index_<file_name>` index 파일도 storage 단위다.
- `SNAPSHOT_NANODB_PATH`: NanoDB snapshot store single JSON 파일 경로
- `SNAPSHOT_JFS_PATH`: jfs snapshot store single JSON 파일 경로
- `SNAPSHOT_JSON_STORE_PATH`: json_store snapshot store append-only JSON line 파일 경로
- `SNAPSHOT_JSON_DB_RS_PATH`: json_db_rs snapshot store JSON event log 파일 경로
- `SNAPSHOT_CDB64_PATH`: cdb64 snapshot store single-file CDB 경로
- `SNAPSHOT_JSON_MUTEX_DB_PATH`: `SNAPSHOT_STORE=json_mutex_db`일 때 snapshot json-mutex-db JSON 파일 경로
- `SNAPSHOT_TOILETDB_PATH`: `SNAPSHOT_STORE=toiletdb`일 때 snapshot ToiletDB JSON 파일 경로
- `SNAPSHOT_DIR_CACHE_PATH`: `SNAPSHOT_STORE=dir_cache`일 때 snapshot dir-cache 디렉터리 경로
- `SNAPSHOT_LMDB_RS_CORE_PATH`: `SNAPSHOT_STORE=lmdb_rs_core`일 때 snapshot lmdb-rs-core environment 디렉터리 경로
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
| 단일 노드 재시작 복구만 필요하고 파일 단위 백업/교체가 중요하나 | `file`, `agdb`, `amandine`, `apex_store`, `armdb`, `assystem`, `colon_db`, `citadeldb`, `jammdb`, `persy`, `native_db`, `nikidb`, `nodb`, `redb`, `crepedb`, `rskey`, `rustbreak`, `btree_store`, `structsy`, `abyssiniandb`, `surrealkv`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `sanakirja`, `tinykv`, `vsdb`, `yakv`, `yakvdb`, `saberdb`, `smolldb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `jsondb`, `joydb`, `png_db`, `koit`, `lite_db`, `lmdb_rs_core`, `log_kv`, `mhdb`, `marble`, `loro_kv`, `luckdb`, `deeb`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `mu_db`, `nanodb`, `jfs`, `json_store`, `json_db_rs`, `cdb64`, `json_mutex_db`, `simple_db`, `docdb`, `rcask` 중 단일 path 또는 단일 directory 기반 store를 우선 검토한다. `kopperdb`와 `grebedb`는 디렉터리 기반이지만 각각 단일 root 아래 세그먼트 로그 또는 단일 engine directory만 관리하고, `vsdb`는 `SNAPSHOT_VSDB_PATH/store.meta.json`과 process-global base dir 조합만 관리해 로컬 재기동 복구 절차가 단순한 편이다. |
| 디렉터리 단위 엔진 백업/restore 절차가 더 자연스러운가 | `apex_store`, `armdb`, `flash_kv`, `ghaladb`, `heed`, `hightower_kv`, `hmdb`, `hurrahdb`, `fs_db`, `sqjson`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `kopperdb`, `epoch_db`, `etchdb`, `rumdb`, `fjall`, `parity_db`, `readb`, `kv`, `rustlite`, `rusty_leveldb`, `canopydb`, `ckydb`, `scdb`, `sled`, `yedb`, `siamesedb`, `snaildb`처럼 디렉터리 기반 store를 쓴다. EtchDB도 WAL-backed path store라 디렉터리 전체를 복구 단위로 잡는다. |
| 운영자가 payload를 직접 열어보며 수동 복구해야 하나 | `file`, `pickledb`, `rcask`, `microkv`, `docdb`, `json_store`, `json_db_rs`, `json_mutex_db`가 가장 단순하다. 대신 binary engine보다 payload 크기와 catalog scan 비용을 더 보수적으로 본다. |
| pure-Rust/no-bindgen/no-native-conflict 제약을 현재 빌드 그래프에서 유지해야 하나 | 현재 landed baseline은 `agdb`, `apex_store`, `armdb`, `assystem`, `colon_db`, `flash_kv`, `ghaladb`, `grebedb`, `grumpydb`, `graus_db`, `simple_db`, `docdb`, `eight`, `epoch_db`, `etchdb`, `fastkv`, `ferrumdb`, `rumdb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `dir_cache`, `sanakirja`, `saturn`, `snaildb`, `tinykv`, `vsdb`, `yakv`, `yakvdb`, `saberdb`, `smolldb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `jsondb`, `joydb`, `png_db`, `kopperdb`, `kv`, `koit`, `lite_db`, `lmdb_rs_core`, `log_kv`, `mhdb`, `marble`, `loro_kv`, `luckdb`, `deeb`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `mu_db`, `nanodb`, `jfs`, `json_store`, `json_db_rs`, `cdb64`, `json_mutex_db`, `persistent_kv`, `nikidb`, `nodb`, `readb`, `rustlite`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `crystal`, `scdb`, `skv`, `surrealkv`, `rskey`, `hightower_kv`, `hmdb`, `hurrahdb`, `fs_db`, `sqjson`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `citadeldb`, `nebari`, `rcask`다. `grebedb` upstream는 `rmp-serde` 0.15 계열에 묶여 있어 현재 workspace에서는 path-vendoring 패치가 필요했다. 추가 후보를 검토할 때도 native `links` 충돌과 bindgen 필요 여부를 먼저 배제한다. |

backend별 운영 차이를 빠르게 확인하려면 아래 매트릭스를 기준으로 본다.

| Backend | 저장 단위 | 운영자 payload 가시성 | 손상/복구 주의점 | 제약 메모 |
| --- | --- | --- | --- | --- |
| `file` | 문서별 JSON 파일 | 가장 높음 | 파일 하나 손상 시 해당 문서만 직접 격리 가능 | baseline filesystem store |
| `agdb` | 단일 memory-mapped graph DB 파일 | 중간 | `snapshot:<doc_id>` alias node에 JSON payload를 저장하고 alias catalog scan으로 문서 목록을 복구한다. 단일 파일 backup/restore와 회귀 테스트 기반 검증이 기본 절차다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `amandine` | 디렉터리 + JSON collection 파일 | 높음 | `snapshots.json` collection에 `doc_id -> persisted snapshot JSON` record를 저장한다. whole-file rewrite와 전체 JSON parse에 의존하므로 directory-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `apex_store` | 디렉터리 WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload key와 explicit `__catalog__` key를 ApexStore engine에 저장한다. 엔진 디렉터리 전체가 복구 단위라 directory-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `armdb` | 디렉터리 sharded Bitcask-style VarTree store | 낮음 | UUID bytes key에 persisted snapshot JSON bytes를 저장하고 tree iteration으로 catalog를 복구한다. adapter는 fsync-enabled flush를 호출하지만 엔진 디렉터리 전체 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `grumpydb` | 디렉터리 WAL-backed page/B+Tree object store | 중간 | `data.db`, `index.db`, `wal.log` 파일 세트에 UUID key와 bytes payload를 저장하고 full range scan으로 catalog를 복구한다. adapter는 save/delete 뒤 `flush()`로 checkpoint와 WAL truncate를 수행하지만, 운영 backup/restore는 디렉터리 전체를 단위로 잡고 회귀 테스트로 검증해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `graus_db` | 디렉터리 append-only log store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 저장한다. save/delete 뒤 handle 재오픈으로 buffered writer flush와 log replay 경계를 고정한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `mindb` | 디렉터리 WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload key와 explicit `__catalog__` key를 저장하고 save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있으면 adapter가 upstream `RecoveryManager`로 WAL을 재생한다. directory-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | no-bindgen/no-new-native-conflict 기준선 |
| `mmdb` | 디렉터리 WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload key와 explicit `__catalog__` key를 write batch로 함께 저장하고 sync write 뒤 flush한다. directory-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `mu_db` | data/index 파일 쌍 key-value store | 중간 | `snapshot:<doc_id>` payload key와 explicit `__catalog__` key를 저장하고 save/delete 뒤 data/index 파일을 fsync한다. 백업/복구는 `SNAPSHOT_MU_DB_PATH`와 같은 디렉터리의 `index_<file_name>`을 함께 다뤄야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `heed` | 디렉터리 + LMDB data file | 낮음 | 엔진 파일 단위 백업이 필요하고 수동 entry 복구는 어렵다 | mmap 기반, pure-Rust baseline에는 포함하지 않음 |
| `hightower_kv` | 디렉터리 + log-structured segments/snapshots | 낮음 | `snapshot:<doc_id>` prefix scan을 쓰므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `hmdb` | 디렉터리 + append-only bincode log | 낮음 | schema 로그 replay로 catalog를 복구한다. tail truncation은 incomplete write로 흡수할 수 있지만, 중간 구간 손상이나 스키마 불일치는 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `icefalldb` | 디렉터리 + append-only `rsdb.log` | 낮음 | 공개 delete/iterator API가 없어 `doc_id` tombstone과 explicit `__catalog__` key를 함께 유지한다. restart recovery는 append-only log replay에 의존하므로 디렉터리 전체 backup/restore와 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitask` | 디렉터리 + append-only active/immutable logs | 낮음 | explicit `__catalog__` key를 같은 log에 유지한다. startup에는 log replay로 keydir를 재구축하고, writer lock이 단일 프로세스만 허용되므로 shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitkv_rs` | 디렉터리 + append-only Bitcask-style data files | 낮음 | `doc_id -> persisted snapshot JSON` key-value를 sync write로 저장하고 startup에는 log replay로 in-memory index를 재구축한다. writer lock이 단일 프로세스만 허용되므로 shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `bitcask_engine` | 디렉터리 + append-only Bitcask log files | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 bitcask-engine-rs log에 저장한다. startup에는 log replay로 in-memory index를 재구축하고, adapter가 process-local mutation을 mutex로 직렬화한다. shared multi-writer durability 용도로는 부적합하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `candystore` | 디렉터리 + append-only data/log/index files | 낮음 | large payload는 `set_big/get_big`로 저장하고 `__catalog__` key를 함께 유지한다. `flush`와 `checkpoint` 뒤 durable cursor를 전진시키므로 엔진 디렉터리 전체 백업이 필요하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `celerix_store` | 디렉터리 + persona JSON 파일 | 중간 | `snapshots.json` persona 파일의 `documents` app map에 `doc_id -> persisted snapshot JSON` value를 저장한다. save/delete는 Celerix Store `Persistence::save_persona` write-then-rename 경계를 쓰지만 persona 파일 전체가 복구 단위이므로 디렉터리 백업과 회귀 테스트 기반 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kopperdb` | 디렉터리 + append-only segment logs | 중간 | 공개 delete API가 없어 `doc_id` key를 tombstone string으로 가리고 explicit `__catalog__` key를 같이 유지한다. 운영자는 세그먼트 디렉터리 전체 백업/restore와 restart 회귀 테스트를 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jammdb` | 단일 파일 | 낮음 | bucket 내부 key는 분리되지만 payload는 binary라 수동 복구가 어렵다 | single-file backup에 유리 |
| `mace` | 디렉터리 + log-structured bucket store | 낮음 | `snapshots` bucket explicit catalog key로 catalog를 복구한다. WAL/data/blob 파일 세트를 함께 백업해야 하고, crate가 1.0 전이라 format/API 변동 가능성을 운영 기준에 반영해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `janql` | 디렉터리 + WAL/SSTable | 중간 | `__catalog__` key와 WAL/SSTable 파일 세트를 함께 백업해야 하고, write는 WAL sync 뒤 in-memory table에 반영된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jasondb` | 단일 append-only JSON log 파일 | 중간 | `doc_id -> persisted snapshot JSON string` entry를 저장하고 startup index replay로 catalog를 복구한다. 삭제는 tombstone append 방식이므로 compaction 전까지 로그가 증가하고, file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jasonisnthappy` | 단일 MVCC/WAL DB 파일 | 낮음 | `snapshots` collection에 `_id=<doc_id>` document로 persisted snapshot JSON payload를 저장한다. DB 파일과 sidecar lock/WAL 파일을 같은 복구 단위로 보고 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `fjall` | 디렉터리 keyspace | 낮음 | LSM directory 전체를 함께 백업해야 한다 | directory-backed engine |
| `persy` | 단일 파일 + index | 낮음 | entry 단위 skip은 가능하지만 index 일관성 검증이 필요하다 | single-file engine |
| `persistent_kv` | 디렉터리 + WAL/snapshot set | 낮음 | snapshot 디렉터리 전체와 WAL/shard 파일을 함께 백업해야 하고, payload는 binary value라 수동 수정보다 재시작 복구 검증이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `native_db` | 단일 파일 | 낮음 | primary-key catalog라 payload 직접 점검은 어렵다 | single-file engine |
| `nebari` | 디렉터리 + append-only tree store | 낮음 | `snapshots` tree range scan으로 catalog를 복구하므로 엔진 디렉터리 전체를 함께 백업해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nikidb` | 단일 파일 B+tree bucket store | 낮음 | explicit `__catalog__` key와 문서 payload가 같은 B+tree file에 함께 저장된다. 수동 payload inspection은 어렵지만 bucket upsert와 single-file backup 절차는 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `nodb` | 단일 파일 DB | 중간 | map 전체를 dump/rename 경계로 다시 쓰고 reopen 시 전체 load에 의존하므로 file corruption 시 startup 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `parity_db` | 디렉터리 column store | 낮음 | ordered column 전체를 묶어 관리해야 한다 | directory-backed engine |
| `pickledb` | 단일 JSON 유사 DB 파일 | 높음 | 사람이 읽기 쉽지만 대용량 catalog에서는 scan 비용을 더 보수적으로 본다 | text-oriented store |
| `rcask` | 디렉터리 + append-only log segments | 중간 | `doc_id` payload와 explicit `__catalog__` key를 UTF-8 JSON string으로 저장하고, 공개 delete API가 없어 tombstone string으로 삭제를 가린다. RCask 디렉터리 전체를 backup/restore해야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
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
| `crepedb` | 단일 redb 파일 | 낮음 | CrepeDB basic table에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장한다. redb 파일 단위 백업/restore와 회귀 테스트 기반 검증이 기본 절차다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
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
| `thetadb` | 단일 파일 B+tree KV | 낮음 | cursor full scan으로 catalog를 복구하기 쉽지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dblite` | 단일 파일 append/reuse KV | 중간 | key index는 reopen 시 파일 전체 scan으로 재구성되고 file-level lock에 의존하므로, 단일 파일 백업은 단순하지만 partial file corruption 시 재구성 실패 가능성을 염두에 둬야 한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dbless` | 단일 파일 typed table store | 낮음 | redb-backed typed table이라 수동 payload inspection은 어렵지만 named table key scan은 단순하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `db_rs` | 디렉터리 + append-only typed table log | 낮음 | `LookupTable<String, PersistedSnapshot>`가 append-only bincode log를 replay해 catalog를 재구성하므로 디렉터리 전체 백업이 필요하고, payload는 binary라 수동 수정 대신 회귀 테스트 기반 복구가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dharmadb` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `doc_id` key와 explicit `__catalog__` key를 같은 keyspace에 저장한다. upstream DB 인스턴스가 비-Send라 adapter가 전용 worker thread로 접근을 직렬화하고, 운영자는 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `sanakirja` | 단일 파일 copy-on-write B-tree | 낮음 | full scan catalog는 단순하지만 payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `flash_kv` | 디렉터리 + append-only bitcask-style engine | 낮음 | `__catalog__` key와 active data file sync 경계를 함께 백업해야 하고, payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ghaladb` | 디렉터리 + LSM key/value store와 value log | 중간 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 JSON string record로 저장하고 save/delete 뒤 `sync()`를 호출한다. upstream bincode 2 API 호환성은 vendored patch로 고정하므로 엔진 디렉터리와 vendor patch를 함께 회귀 테스트로 검증한다 | pure-Rust/no-bindgen/no-native-conflict 기준선, vendored bincode API patch |
| `snaildb` | 디렉터리 + WAL/SSTable LSM engine | 낮음 | `__catalog__` key와 엔진 디렉터리를 함께 백업해야 하고, payload는 binary value라 수동 수정보다 entry skip 기반 대응이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `tinykv` | 단일 JSON 파일 store | 중간 | payload 가시성은 가장 높지만 whole-file rewrite와 전체 JSON 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패로 이어질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `yakv` | 단일 B-Tree 파일 | 낮음 | `snapshot:<doc_id>` key를 직접 저장하고 full scan으로 catalog를 복구한다. payload는 binary value이고 파일 전체 무결성에 의존하므로 수동 수정 대신 whole-file backup/restore와 회귀 테스트가 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `saberdb` | 단일 pretty JSON 파일 store | 중간 | atomic temp+rename은 단순하지만 catalog 전체를 pretty JSON으로 다시 쓰고 startup 시 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 된다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `smolldb` | 단일 compressed 파일 store | 낮음 | in-memory key-value map을 zlib-compatible compressed 파일로 temp+rename 저장한다. 전체 파일 load/rewrite 경계라 단일 노드 재시작 복구에 맞고, corruption 시 startup 전체 복구 실패가 될 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kstone` | 디렉터리 + WAL/SSTable LSM store | 낮음 | Kstone item의 binary payload에 `snapshot:<doc_id>` 값과 explicit `__catalog__` key를 저장하고 save/delete 뒤 flush한다. 엔진 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `roughdb` | 디렉터리 + LevelDB-compatible WAL/SSTable store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 sync write batch로 함께 저장하고 wait flush로 재시작 복구 경계를 고정한다. 엔진 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | no-bindgen/no-new-native-conflict 기준선 |
| `raindb` | 디렉터리 + LevelDB-style WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 synchronous batch로 함께 저장한다. 교육용 LevelDB port 성격이 강하므로 엔진 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `infusedb` | 단일 document-oriented text 파일 store | 중간 | `snapshots` collection에 base64-encoded `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 저장한다. whole-file dump/load 경계라 수동 payload inspection은 가능하지만 파일 손상 시 startup 전체 복구 실패가 될 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `kafi` | 단일 bincode hashmap 파일 store | 낮음 | `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 저장한다. save/delete마다 whole-file truncate/write를 수행하므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ledger_kv` | 디렉터리 + append-only ledger/bin/meta files | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 ledger-kv journal에 저장하고 data/meta 파일을 sync한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `blazeup` | 디렉터리 + kv/sled bucket store | 낮음 | blazeup의 process-global path 설정을 adapter mutex로 직렬화하고 `snapshots` bucket에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 JSON string record로 저장한다. 내부 엔진은 `kv`/sled라 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `feoxdb` | 단일 FeOxDB 파일 append event store | 중간 | mutable same-key update replay를 피하기 위해 `snapshot:<doc_id>:<timestamp>:<event_id>` immutable event key와 tombstone event만 쓴다. range scan으로 최신 event를 복구하며 기본 jemalloc feature는 끄고 `system-alloc`으로 연결한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `jsondb` | 단일 versioned pretty JSON 파일 store | 중간 | write guard drop마다 whole-file pretty JSON rewrite와 전체 역직렬화에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `png_db` | 단일 PNG compressed text chunk 파일 | 중간 | `doc_id`와 persisted snapshot JSON payload를 PNG zTXt row chunk로 저장하고 save/delete마다 전체 row set을 temp PNG로 교체한다. 파일 단위 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `koit` | 단일 structured JSON 파일 store | 중간 | 전체 catalog를 메모리에 로드한 뒤 save마다 whole-file rewrite와 `sync_all`을 수행하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lite_db` | 디렉터리 + append-only data files | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 sync write로 저장한다. file lock이 단일 writer를 전제하므로 shared multi-writer durability나 authoritative coordination plane으로는 쓰지 않는 것이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lmdb_rs_core` | 디렉터리 + LMDB-style B+tree environment | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 main database에 저장하고 commit 뒤 forced sync로 복구 경계를 고정한다. environment 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | no-bindgen/no-new-native-conflict 기준선 |
| `append_kv` | 단일 append-only log 파일 | 중간 | vendored lib target이 노출한 `KvStore`에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 JSON string으로 저장한다. save/delete마다 file sync를 수행하지만 compaction이 없어 파일은 계속 커질 수 있으므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선, vendored lib target patch |
| `mhdb` | DBM path prefix + `.pag`/`.dir` 파일 쌍 | 낮음 | upstream pair size 제한이 506B라 persisted snapshot JSON bytes와 catalog를 chunked blob key로 나눠 저장한다. `<path>.pag`/`<path>.dir` 파일 쌍 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `luckdb` | 단일 JSON document DB 파일 | 중간 | LuckDB `backend.snapshots` collection에 `doc_id`와 persisted snapshot JSON payload를 함께 저장한다. collection 전체 query로 catalog를 복구하므로 단일 파일 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ipjdb` | 디렉터리 + collection별 JSON item 파일 | 중간 | ipjdb `snapshots` collection에 `doc_id`와 persisted snapshot JSON payload를 함께 저장한다. collection full scan으로 catalog를 복구하므로 directory-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선, upstream maintenance 주의 |
| `deeb` | 단일 JSON database 파일 | 중간 | Deeb `snapshots` entity에 `doc_id` primary key와 persisted snapshot JSON payload를 함께 저장한다. save/delete마다 temp+rename commit을 수행하므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `rubin` | 단일 JSON key-value 파일 | 중간 | Rubin `MemStore` string map에 `doc_id -> persisted snapshot JSON` payload를 저장한다. save/delete마다 whole-file JSON rewrite를 수행하므로 단일 파일 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `etchdb` | 디렉터리 + WAL-backed path store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 EtchDB WAL-backed store에 저장하고 `write_durable`로 save/delete를 fsync한다. 엔진 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `fastkv` | 단일 compressed binary dump 파일 | 낮음 | FastKV in-memory key/value store에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고 save/delete마다 temp dump를 fsync한 뒤 rename한다. 파일 단위 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lsm_engine` | 단일 WAL 파일 + in-memory LSM rebuild | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 lsm_engine WAL에 JSON string으로 저장하고 reopen 때 WAL replay로 memtable을 재구성한다. 단일 WAL 파일 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선, vendored serde import patch |
| `lsm_storage_engine` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 WAL-first engine에 저장하고 save/delete 뒤 flush로 복구 경계를 고정한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lsmdb` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 WAL-first engine에 저장하고 WAL sync-on-write 경계로 복구를 고정한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `lsm_tree` | 디렉터리 + flush-only LSM-tree primitive | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고 save/delete 뒤 active memtable을 flush해 복구 경계를 고정한다. WAL을 제공하지 않는 primitive라 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `ferrumdb` | 단일 append-only log 파일 store | 낮음 | JSON value를 `snapshot:<doc_id>` payload와 explicit `__catalog__` key로 append하고 `FsyncPolicy::Always`로 write마다 sync한다. compaction 전까지 로그가 증가하므로 file-level backup/restore와 회귀 테스트 기반 검증을 기본 절차로 둔다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `mindb` | 디렉터리 + WAL/SSTable LSM store | 낮음 | `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고 save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있으면 adapter가 upstream `RecoveryManager`로 WAL을 재생한다. 디렉터리 전체 백업/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다 | no-bindgen/no-new-native-conflict 기준선 |
| `jfs` | 단일 JSON object store | 높음 | single-file catalog를 temp+rename으로 교체해 각 `doc_id` object를 저장한다. payload inspection은 쉽지만 whole-file parse에 의존하므로 파일 손상 시 startup 전체 복구 실패가 전체 store에 번질 수 있다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `json_store` | 단일 append-only JSON line 파일 store | 높음 | key별 최신 line offset을 메모리 인덱스로 유지하므로 payload inspection은 쉽지만, compaction 없이는 append log가 계속 커지고 startup catalog rebuild는 전체 파일 replay에 의존한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `json_mutex_db` | 단일 JSON object store | 높음 | root object에 `doc_id -> persisted snapshot JSON` entry를 저장하고 save/delete마다 atomic temp+rename으로 전체 파일을 교체한다. payload inspection은 쉽지만 startup catalog rebuild는 whole-file parse에 의존한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
| `dir_cache` | 디렉터리 기반 cache key store | 중간 | `snapshot-<doc_id>.json` payload와 `__catalog__` key를 dir-cache entry로 저장하고 save/delete 뒤 `sync()`로 디렉터리 상태를 flush한다. 공개 key iterator가 없어 explicit catalog key에 의존한다 | pure-Rust/no-bindgen/no-native-conflict 기준선 |
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
- `hurrahdb`는 append-only file-backed in-memory key-value embedded durability 기준선이다.
- `fs_db`는 key-per-file JSON directory-backed embedded durability 기준선이다.
- `sqjson`은 single-file JSON DB embedded durability 기준선이다. adapter가 page당 payload 한계를 피하려고 persisted snapshot JSON을 base64 chunk key로 나눠 저장하지만, upstream index page 한계가 있어 매우 큰 document set에서는 보수적으로 검증해야 한다.
- `surrealkv`는 single-file embedded durability 기준선이다.
- `thunderdb`는 single-file embedded durability 기준선이다.
- `thetadb`는 single-file B+tree embedded durability 기준선이다.
- `dbless`는 redb-backed single-file typed table embedded durability 기준선이다.
- `sanakirja`는 single-file copy-on-write B-tree embedded durability 기준선이다.
- `flash_kv`는 append-only directory-backed embedded durability 기준선이다.
- `snaildb`는 WAL/SSTable directory-backed embedded durability 기준선이다.
- `tinykv`는 human-readable single-file embedded durability 기준선이다.
- `yakv`는 single-file B-Tree embedded durability 기준선이다.
- `saberdb`는 atomic temp+rename pretty JSON embedded durability 기준선이다.
- `smolldb`는 in-memory key-value map을 compressed single-file backup으로 flush하는 embedded durability 기준선이다.
- `kstone`은 WAL/SSTable LSM 디렉터리 저장소를 쓰는 embedded durability 기준선이다.
- `roughdb`는 LevelDB-compatible WAL/SSTable 디렉터리 저장소를 sync write batch와 flush 경계로 연결한 embedded durability 기준선이다.
- `jsondb`는 schema-versioned pretty JSON embedded durability 기준선이다.
- `lite_db`는 LiteDb append-only 디렉터리 저장소를 쓰면서도 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `lsm_engine`은 WAL replay 기반 LSM 저장소를 쓰면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다. upstream `serde::export` import는 현재 serde와 맞지 않아 path-vendoring으로 `std::convert::TryFrom`만 패치했다.
- `lsm_storage_engine`은 zero-dependency WAL/SSTable LSM 디렉터리 저장소를 쓰면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `lsmdb`는 WAL/SSTable LSM 디렉터리 저장소를 쓰면서 현재 저장소 제약(pure-Rust/no-bindgen/no-native-conflict)을 유지한 추가 기준선이다.
- `mindb`는 WAL/SSTable LSM 디렉터리 저장소를 `CompressionCodec::None`, explicit `sync()` 경계, reopen WAL replay fallback으로 연결한 추가 기준선이다.
- `koit`는 async whole-file structured JSON embedded durability 기준선이다.
- `jfs`는 single-file JSON object embedded durability 기준선이다.
- `toiletdb`는 temp file persist 기반 single-file JSON embedded durability 기준선이다.
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
- corrupt entry를 warning과 함께 건너뛰는 현재 catalog 정책을 적극 활용하려면 `flash_kv`, `hightower_kv`, `jammdb`, `persy`, `native_db`, `redb`, `btree_store`, `siamesedb`, `abyssiniandb`, `ckydb`, `crepedb`, `crystal`, `scdb`, `skv`, `surrealkv`, `thunderdb`, `dblite`, `dbless`, `sanakirja`, `saturn`, `snaildb`, `simple_db` 쪽이 기본값 후보로 더 안전하다. `hmdb`는 tail truncation은 흡수할 수 있지만 로그 중간 손상 시 startup 전체 복구 실패로 이어질 수 있고, `nikidb`는 single-file bucket store라 backup 절차는 단순하지만 binary B+tree file 전체 무결성에 더 의존한다.
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
- `SNAPSHOT_STORE=jammdb`는 `SNAPSHOT_JAMMDB_PATH` 단일 jammdb 파일을 통해 vendor-specific embedded database durability를 사용한다. `SNAPSHOT_STORE=mace`는 `SNAPSHOT_MACE_PATH` Mace 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
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
- `SNAPSHOT_STORE=etchdb`는 `SNAPSHOT_ETCHDB_PATH` EtchDB WAL-backed 디렉터리에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고 `write_durable`로 save/delete 복구 경계를 고정한다.
- `SNAPSHOT_STORE=fastkv`는 `SNAPSHOT_FASTKV_PATH` FastKV compressed binary dump 파일에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key를 저장하고 save/delete마다 temp dump rename으로 복구 경계를 고정한다.
- snapshot payload는 rumdb keyspace의 `doc_id` key에 저장되고, document catalog는 explicit `__catalog__` key와 startup log replay 결과로 복구된다.
- `SNAPSHOT_STORE=rusty_leveldb`는 `SNAPSHOT_RUSTY_LEVELDB_PATH` rusty-leveldb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rusty-leveldb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, `GET /api/documents` catalog는 same keyspace full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- `SNAPSHOT_STORE=canopydb`는 `SNAPSHOT_CANOPYDB_PATH` canopydb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 canopydb `snapshots` tree에 `doc_id -> persisted snapshot JSON` key-value로 저장된다.
- `SNAPSHOT_STORE=ckydb`는 `SNAPSHOT_CKYDB_PATH` ckydb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 ckydb key-value 엔트리에 `doc_id -> base64(persisted snapshot JSON)`로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=crepedb`는 `SNAPSHOT_CREPEDB_PATH` 단일 CrepeDB redb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 CrepeDB basic table의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, document catalog는 같은 table의 explicit `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=crystal`은 `SNAPSHOT_CRYSTAL_PATH` 디렉터리의 `<doc_id>.bin` key-per-file 저장소를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 crystal file value에 persisted snapshot JSON string으로 저장되고, document catalog는 디렉터리 스캔으로 복구된다.
- `SNAPSHOT_STORE=scdb`는 `SNAPSHOT_SCDB_PATH` scdb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 scdb key-value 엔트리에 `doc_id -> persisted snapshot JSON`으로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=skv`는 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data` / `<path>.index` 파일 쌍을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 skv key-value 엔트리에 `doc_id -> persisted snapshot JSON`으로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=surrealkv`는 `SNAPSHOT_SURREALKV_PATH` 단일 surrealkv B+tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 surrealkv keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- 손상된 snapshot payload는 `GET /api/documents` catalog 생성 중 warning과 함께 건너뛰고, 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=pickledb`는 `SNAPSHOT_PICKLEDB_PATH` 단일 PickleDB 파일을 통해 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=agdb`는 `SNAPSHOT_AGDB_PATH` 단일 agdb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 agdb의 `snapshot:<doc_id>` alias node payload key에 JSON string으로 저장되고, document catalog는 all-alias scan 뒤 matching alias node를 다시 읽어 복구된다.
- `SNAPSHOT_STORE=amandine`는 `SNAPSHOT_AMANDINE_PATH` 디렉터리의 `snapshots.json` collection을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Amandine `snapshots` collection의 `doc_id -> persisted snapshot JSON` record로 저장되고, document catalog는 `snapshots.json` whole-file parse로 복구된다.
- `SNAPSHOT_STORE=apex_store`는 `SNAPSHOT_APEX_STORE_PATH` 디렉터리의 ApexStore WAL/SSTable LSM engine을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 ApexStore engine의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 persisted snapshot JSON bytes로 저장되고, document catalog는 WAL replay 뒤 catalog key를 읽어 복구된다.
- `SNAPSHOT_STORE=armdb`는 `SNAPSHOT_ARMDB_PATH` 디렉터리의 sharded ArmDB VarTree를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 ArmDB VarTree에 UUID bytes key와 persisted snapshot JSON bytes value로 저장되고, document catalog는 tree iteration 뒤 각 payload를 복원해 구성된다.
- `SNAPSHOT_STORE=rcask`는 `SNAPSHOT_RCASK_PATH` RCask append-only segment 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 rcask segment log의 `doc_id` key와 explicit `__catalog__` key에 JSON string으로 저장되고, delete는 tombstone string을 덮어써 가린다. document catalog는 같은 `__catalog__` key를 읽어 복구된다.
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
- `SNAPSHOT_STORE=cacache`는 `SNAPSHOT_CACACHE_PATH` content-addressed cache 디렉터리를 통해 vendor-specific embedded cache durability를 사용한다.
- snapshot payload는 cacache index의 `snapshot:<doc_id>` key에 `persisted snapshot JSON` bytes로 저장된다.
- 기본 local ownership 모드에서는 startup cache index listing 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
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
- `SNAPSHOT_STORE=thetadb`는 `SNAPSHOT_THETADB_PATH` 단일 thetadb 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 thetadb keyspace의 raw `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 cursor full scan으로 복구된다.
- `SNAPSHOT_STORE=grebedb`는 `SNAPSHOT_GREBEDB_PATH` 단일 grebedb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 grebedb keyspace의 `doc_id` key에 저장되고, document catalog는 explicit `__catalog__` key로 유지된다. save/delete는 payload와 catalog를 같은 `flush()` 경계에서 반영한다.
- `SNAPSHOT_STORE=grumpydb`는 `SNAPSHOT_GRUMPYDB_PATH` 단일 grumpydb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 GrumpyDB UUID key에 bytes value로 저장되고, document catalog는 full range scan으로 복구된다. WAL 모듈은 아직 upstream roadmap 단계라 운영자는 디렉터리 전체 backup/restore와 회귀 테스트 기반 검증을 기본 절차로 보는 편이 안전하다.
- `SNAPSHOT_STORE=graus_db`는 `SNAPSHOT_GRAUS_DB_PATH` GrausDb append-only log 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 GrausDb keyspace의 `doc_id -> persisted snapshot JSON` value와 explicit `__catalog__` key로 저장되고, save/delete 뒤 handle 재오픈으로 normal restart 복구 경계를 고정한다.
- `SNAPSHOT_STORE=vsdb`는 `SNAPSHOT_VSDB_PATH/store.meta.json`에 store handle metadata를 두고, upstream `vsdb`의 process-global base dir(`VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb`)을 실제 durability keyspace로 사용한다.
- snapshot payload는 vsdb `Mapx<String, Vec<u8>>`의 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 same keyspace full scan으로 복구된다. 서버는 snapshot store 접근을 직렬화해 concurrent mutation을 제한한다.
- `SNAPSHOT_STORE=dblite`는 `SNAPSHOT_DBLITE_PATH` 단일 dblite 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 dblite string key-value 엔트리에 `doc_id -> persisted snapshot JSON` bytes로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=dbless`는 `SNAPSHOT_DBLESS_PATH` 단일 dbless 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 dbless typed table 엔트리에 `doc_id -> persisted snapshot`으로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=db_rs`는 `SNAPSHOT_DB_RS_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=dharmadb`는 `SNAPSHOT_DHARMADB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 db-rs `LookupTable<String, PersistedSnapshot>` 엔트리에 `doc_id -> persisted snapshot`으로 저장되고, document catalog는 append-only log replay 뒤 same table scan으로 복구된다.
- `SNAPSHOT_STORE=sanakirja`는 `SNAPSHOT_SANAKIRJA_PATH` 단일 sanakirja copy-on-write B-tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 sanakirja keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- `SNAPSHOT_STORE=snaildb`는 `SNAPSHOT_SNAILDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 snaildb keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 별도 `__catalog__` key로 유지된다.
- `SNAPSHOT_STORE=tinykv`는 `SNAPSHOT_TINYKV_PATH` 단일 tinykv JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 tinykv keyspace에 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 key scan으로 복구된다.
- `SNAPSHOT_STORE=yakv`는 `SNAPSHOT_YAKV_PATH` 단일 yakv B-Tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 yakv keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, document catalog는 full scan으로 복구된다.
- `SNAPSHOT_STORE=yakvdb`는 `SNAPSHOT_YAKVDB_PATH` 단일 yakvdb B-Tree 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 yakvdb keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` key-value로 저장되고, document catalog는 min/above key traversal로 복구된다.
- `SNAPSHOT_STORE=saberdb`는 `SNAPSHOT_SABERDB_PATH` 단일 saberdb pretty JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 saberdb catalog에 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=smolldb`는 `SNAPSHOT_SMOLLDB_PATH` 단일 compressed SmollDB 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON bytes` key-value와 explicit `__catalog__` key로 저장되고, document catalog는 file load 뒤 catalog key로 복구된다.
- `SNAPSHOT_STORE=kstone`는 `SNAPSHOT_KSTONE_PATH` Kstone WAL/SSTable LSM 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Kstone item의 binary field에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` key-value와 explicit `__catalog__` key로 저장되고, document catalog는 catalog key 뒤 각 payload를 복원해 복구된다.
- `SNAPSHOT_STORE=roughdb`는 `SNAPSHOT_ROUGHDB_PATH` RoughDB WAL/SSTable 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 roughdb keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` 값과 explicit `__catalog__` key로 저장되고, save/delete는 sync write batch와 wait flush로 확정된다.
- `SNAPSHOT_STORE=raindb`는 `SNAPSHOT_RAINDB_PATH` RainDB WAL/SSTable 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 raindb keyspace에 `snapshot:<doc_id> -> persisted snapshot JSON` 값과 explicit `__catalog__` key로 저장되고, save/delete는 synchronous write batch로 확정된다.
- `SNAPSHOT_STORE=infusedb`는 `SNAPSHOT_INFUSEDB_PATH` InfuseDB 단일 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 InfuseDB `snapshots` collection에 base64 text로 인코딩된 `snapshot:<doc_id> -> persisted snapshot JSON bytes` 값과 explicit `__catalog__` key로 저장되고, save/delete는 whole-file dump로 확정된다.
- `SNAPSHOT_STORE=kafi`는 `SNAPSHOT_KAFI_PATH` kafi 단일 파일을 통해 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=tinkv`는 `SNAPSHOT_TINKV_PATH` tinkv append-only data 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 kafi bincode hashmap 파일에 `snapshot:<doc_id> -> persisted snapshot JSON string` 값과 explicit `__catalog__` key로 저장되고, save/delete는 whole-file flush로 확정된다.
- `SNAPSHOT_STORE=ledger_kv`는 `SNAPSHOT_LEDGER_KV_PATH` ledger-kv append-only ledger 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 ledger-kv journal에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` 값과 explicit `__catalog__` key로 저장되고, save/delete는 data/meta file sync로 확정된다.
- `SNAPSHOT_STORE=feoxdb`는 `SNAPSHOT_FEOXDB_PATH` 단일 FeOxDB 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id>:<timestamp>:<event_id> -> persisted snapshot JSON` immutable event와 tombstone event로 저장되고, document catalog는 prefix range scan 뒤 최신 event 선택으로 복구된다.
- `SNAPSHOT_STORE=jsondb`는 `SNAPSHOT_JSONDB_PATH` 단일 jsondb versioned pretty JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 jsondb catalog의 `snapshots.<doc_id>` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=joydb`는 `SNAPSHOT_JOYDB_PATH` 단일 Joydb JSON state 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Joydb `JoydbSnapshotRecord` catalog에 `doc_id -> persisted snapshot` record로 저장되고, save/delete 뒤 `flush()`로 document catalog를 복구한다.
- `SNAPSHOT_STORE=png_db`는 `SNAPSHOT_PNG_DB_PATH` 단일 PNG 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 png-db compressed text row chunk에 `doc_id`와 persisted snapshot JSON payload를 함께 저장하고, save/delete마다 temp PNG rename으로 document catalog를 복구한다.
- `SNAPSHOT_STORE=kv`는 `SNAPSHOT_KV_PATH` sled 디렉터리와 `snapshots` bucket을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 kv catalog의 `doc_id -> persisted snapshot JSON` key-value로 저장되고, document catalog는 same bucket full scan으로 복구된다.
- `SNAPSHOT_STORE=eight`는 `SNAPSHOT_EIGHT_PATH` 디렉터리 아래 eight filesystem storage를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 eight keyspace의 `doc_<uuid_simple> -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 empty-prefix search 결과를 다시 load해 복구된다.
- `SNAPSHOT_STORE=lite_db`는 `SNAPSHOT_LITE_DB_PATH` LiteDb 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key로 저장되고, document catalog는 catalog key 뒤 각 payload를 다시 읽어 복구된다.
- `SNAPSHOT_STORE=log_kv`는 `SNAPSHOT_LOG_KV_PATH` append-only 단일 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key로 저장되고, delete는 tombstone string으로 가린다.
- `SNAPSHOT_STORE=append_kv`는 `SNAPSHOT_APPEND_KV_PATH` append-only 단일 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key로 저장되고, delete는 append_kv tombstone record로 가린다.
- `SNAPSHOT_STORE=append_log`는 `SNAPSHOT_APPEND_LOG_PATH` append-only 단일 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload와 delete는 save/delete JSON event로 append되고, startup hydrate/list 경로는 event log replay로 복구된다.
- `SNAPSHOT_STORE=mhdb`는 `SNAPSHOT_MHDB_PATH` path prefix가 만드는 `<path>.pag`/`<path>.dir` DBM 파일 쌍을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload와 catalog는 MHdb pair size 제한을 피하도록 chunked blob key로 나눠 저장된다.
- `SNAPSHOT_STORE=luckdb`는 `SNAPSHOT_LUCKDB_PATH` 단일 LuckDB JSON document 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 LuckDB `backend.snapshots` collection의 JSON document에 저장되고, `doc_id` field query로 load/delete 및 catalog 복구를 수행한다.
- `SNAPSHOT_STORE=ipjdb`는 `SNAPSHOT_IPJDB_PATH` 디렉터리 아래 ipjdb `snapshots` collection item 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `doc_id` field와 persisted snapshot JSON을 함께 가진 item으로 저장되고, collection full scan으로 load/delete 및 catalog 복구를 수행한다.
- `SNAPSHOT_STORE=kagi`는 `SNAPSHOT_KAGI_PATH` 단일 kagi bincode hashmap 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `doc_id -> persisted snapshot JSON string` entry로 저장되고, whole-file map load로 load/delete 및 catalog 복구를 수행한다. upstream panic 기반 I/O는 adapter가 `StorageError`로 매핑한다.
- `SNAPSHOT_STORE=deeb`는 `SNAPSHOT_DEEB_PATH` 단일 Deeb JSON database 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Deeb `snapshots` entity의 `doc_id` primary key record에 저장되고, entity full scan으로 load/delete 및 catalog 복구를 수행한다.
- `SNAPSHOT_STORE=rubin`은 `SNAPSHOT_RUBIN_PATH` 단일 Rubin JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Rubin `MemStore` string map의 `doc_id -> persisted snapshot JSON` entry로 저장되고, document catalog는 string map scan 뒤 각 payload를 다시 읽어 복구된다.
- `SNAPSHOT_STORE=lsm_engine`는 `SNAPSHOT_LSM_ENGINE_PATH` WAL 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key로 저장되고, reopen 때 WAL replay로 document catalog를 복구한다.
- `SNAPSHOT_STORE=lsm_storage_engine`는 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH` WAL/SSTable LSM 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key로 저장되고, save/delete 뒤 flush해 document catalog를 복구한다.
- `SNAPSHOT_STORE=lsmdb`는 `SNAPSHOT_LSMDB_PATH` WAL/SSTable LSM 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key로 저장되고, WAL sync-on-write 경계로 document catalog를 복구한다.
- `SNAPSHOT_STORE=lsm_tree`는 `SNAPSHOT_LSM_TREE_PATH` lsm-tree primitive 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON bytes` key-value와 explicit `__catalog__` key로 저장되고, save/delete 뒤 active memtable flush 경계로 document catalog를 복구한다.
- `SNAPSHOT_STORE=ferrumdb`는 `SNAPSHOT_FERRUMDB_PATH` append-only log 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` JSON value와 explicit `__catalog__` key로 저장되고, `FsyncPolicy::Always`로 write마다 sync해 document catalog를 복구한다.
- `SNAPSHOT_STORE=mindb`는 `SNAPSHOT_MINDB_PATH` WAL/SSTable LSM 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key로 저장되고, save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있으면 adapter가 upstream `RecoveryManager`로 WAL을 재생해 document catalog를 복구한다.
- `SNAPSHOT_STORE=mmdb`는 `SNAPSHOT_MMDB_PATH` WAL/SSTable LSM 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 write batch로 함께 저장하고, sync write 뒤 flush해 document catalog를 복구한다.
- `SNAPSHOT_STORE=nanodb`는 `SNAPSHOT_NANODB_PATH` 단일 NanoDB JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 root object의 `doc_id -> persisted snapshot JSON` entry로 저장되고, save/delete 뒤 whole-file write로 document catalog를 복구한다.
- `SNAPSHOT_STORE=koit`는 `SNAPSHOT_KOIT_PATH` 단일 koit structured JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 koit catalog의 `snapshots.<doc_id>` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=jfs`는 `SNAPSHOT_JFS_PATH` 단일 jfs JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 jfs catalog의 `doc_id -> persisted snapshot JSON string` key-value로 저장되고, document catalog는 whole-file map load로 복구된다.
- `SNAPSHOT_STORE=json_store`는 `SNAPSHOT_JSON_STORE_PATH` 단일 append-only JSON line 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 json_store catalog의 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 whole-file line replay와 key별 최신 offset 인덱스로 복구된다.
- `SNAPSHOT_STORE=fastkv`는 `SNAPSHOT_FASTKV_PATH` 단일 compressed binary dump 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장되고, save/delete마다 temp dump fsync 뒤 rename으로 document catalog를 복구한다.
- `SNAPSHOT_STORE=json_mutex_db`는 `SNAPSHOT_JSON_MUTEX_DB_PATH` 단일 JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 root object의 `doc_id -> persisted snapshot JSON` entry로 저장되고, save/delete마다 json-mutex-db atomic save로 전체 파일을 교체해 document catalog를 복구한다.
- `SNAPSHOT_STORE=toiletdb`는 `SNAPSHOT_TOILETDB_PATH` 단일 JSON 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 root object의 `doc_id -> persisted snapshot JSON` entry로 저장되고, save/delete마다 ToiletDB temp file persist 뒤 file sync로 document catalog를 복구한다.
- `SNAPSHOT_STORE=dir_cache`는 `SNAPSHOT_DIR_CACHE_PATH` 디렉터리의 dir-cache entry set을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot-<doc_id>.json` key와 explicit `__catalog__` key로 저장되고, save/delete마다 `sync()`를 호출해 document catalog를 복구한다.
- `SNAPSHOT_STORE=marble`는 `SNAPSHOT_MARBLE_PATH` 디렉터리의 Marble object store를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 per-document object로 저장되고, document catalog는 fixed catalog object의 `doc_id -> object_id` mapping으로 복구된다. save/delete는 Marble atomic write batch와 `fsync_each_batch=true`로 확정한다.
- `SNAPSHOT_STORE=lmdb_rs_core`는 `SNAPSHOT_LMDB_RS_CORE_PATH` 디렉터리의 lmdb-rs-core environment를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장되고, save/delete commit 뒤 forced sync로 document catalog와 재시작 복구 경계를 고정한다.
- `SNAPSHOT_STORE=loro_kv`는 `SNAPSHOT_LORO_KV_PATH` 단일 binary SSTable 파일을 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 loro-kv-store `MemKvStore`의 `doc_id -> persisted snapshot JSON bytes` key-value로 저장되고, save/delete마다 whole-store export를 temp+rename으로 확정해 document catalog를 복구한다.
- `SNAPSHOT_STORE=hmdb`는 `SNAPSHOT_HMDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 hmdb schema 로그의 `doc_id -> persisted snapshot` key-value로 저장되고, document catalog는 append-only 로그 replay로 복구된다.
- `SNAPSHOT_STORE=hurrahdb`는 `SNAPSHOT_HURRAHDB_PATH` 단일 AOF 파일을 통해 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=fs_db`는 `SNAPSHOT_FS_DB_PATH` 디렉터리의 `snapshot-<doc_id>.json` 파일로 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=sqjson`는 `SNAPSHOT_SQJSON_PATH` 단일 sqjson DB 파일에 `snapshot:<doc_id>` blob을 page-safe chunk key로 나눠 저장하는 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 `snapshot:<doc_id> -> persisted snapshot` key-value로 저장되고, document catalog는 explicit `__catalog__` key와 AOF replay로 복구된다.
- `SNAPSHOT_STORE=icefalldb`는 `SNAPSHOT_ICEFALLDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 icefalldb `rsdb.log`의 `doc_id` key와 explicit `__catalog__` key에 저장되고, delete는 tombstone value를 덮어써 가린다. document catalog는 같은 `__catalog__` key를 읽어 복구된다.
- `SNAPSHOT_STORE=bitask`는 `SNAPSHOT_BITASK_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 bitask append-only log의 `doc_id` key와 explicit `__catalog__` key에 저장되고, document catalog는 startup log replay 뒤 재구축된 keydir를 따라 복구된다.
- `SNAPSHOT_STORE=bitkv_rs`는 `SNAPSHOT_BITKV_RS_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 bitkv-rs append-only log의 `doc_id -> persisted snapshot JSON` key-value 엔트리에 sync write로 저장되고, document catalog는 startup log replay 뒤 재구축된 in-memory index를 따라 복구된다.
- `SNAPSHOT_STORE=bitcask_engine`는 `SNAPSHOT_BITCASK_ENGINE_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- `SNAPSHOT_STORE=blazeup`는 `SNAPSHOT_BLAZEUP_PATH` 디렉터리를 통해 blazeup/kv/sled 기반 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 bitcask-engine-rs append-only log의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 저장되고, document catalog는 startup log replay 뒤 재구축된 in-memory index를 따라 복구된다.
- `SNAPSHOT_STORE=candystore`는 `SNAPSHOT_CANDYSTORE_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 candystore keyspace의 `doc_id` key와 explicit `__catalog__` key에 저장되고, large payload는 `set_big/get_big`로 읽고 쓴 뒤 `flush`와 `checkpoint`로 durable cursor를 전진시킨다.
- `SNAPSHOT_STORE=celerix_store`는 `SNAPSHOT_CELERIX_STORE_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 Celerix Store persistence persona 파일 `snapshots.json` 안의 `documents` app map에 `doc_id -> persisted snapshot JSON` value로 저장되고, document catalog는 같은 app map key를 순회해 복구된다.
- `SNAPSHOT_STORE=kopperdb`는 `SNAPSHOT_KOPPERDB_PATH` 디렉터리를 통해 vendor-specific embedded database durability를 사용한다.
- snapshot payload는 kopperdb append-only 세그먼트의 `doc_id` key와 explicit `__catalog__` key에 저장되고, delete는 tombstone value를 덮어써 가린다. document catalog는 같은 `__catalog__` key를 읽어 복구된다.
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
- 현재 저장소에는 filesystem rehearsal용 coordination surface, SQLite-backed authoritative coordination surface, vendor-specific embedded DB durability surface(`SNAPSHOT_STORE=agdb|heed|hightower_kv|hmdb|jammdb|mace|fjall|persy|persistent_kv|native_db|nebari|nikidb|nodb|parity_db|pickledb|rcask|microkv|redb|rskey|readb|kv|eight|epoch_db|etchdb|fastkv|ferrumdb|rumdb|rustlite|rusty_leveldb|canopydb|caves|ckydb|crepedb|scdb|skv|surrealkv|sled|rustbreak|yedb|btree_store|cacache|siamesedb|structsy|abyssiniandb|aeternusdb|thunderdb|dblite|dbless|db_rs|dharmadb|sanakirja|snaildb|tinykv|yakv|yakvdb|saberdb|smolldb|kstone|jsondb|joydb|png_db|kopperdb|koit|lite_db|lmdb_rs_core|log_kv|mhdb|marble|loro_kv|luckdb|deeb|rubin|lsm_engine|lsm_storage_engine|lsmdb|lsm_tree|mindb|mmdb|nanodb|jfs|json_store|toiletdb|simple_db|docdb|shorterdb|celerix_store|citadeldb|ledger_kv|blazeup`), S3-compatible object storage durability surface, external lease service를 쓰는 managed coordination surface, 그리고 external snapshot service를 쓰는 managed durability surface가 함께 있다. shared snapshot durability 후보로 `SNAPSHOT_STORE=sqlite`를 쓸 수 있고, object storage durability 후보로 `SNAPSHOT_STORE=s3`를 쓸 수 있으며, 외부 service durability 후보로 `SNAPSHOT_STORE=managed`를 쓸 수 있다. 이를 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite` 또는 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`에 연결해 owner lease와 snapshot durability를 분리 구성할 수 있고, managed-managed actual handoff rehearsal까지 회귀 테스트로 검증됐다.

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
10. 재시작 복구를 검증하려면 `SNAPSHOT_STORE=file`, `SNAPSHOT_STORE=agdb`, `SNAPSHOT_STORE=apex_store`, `SNAPSHOT_STORE=sqlite`, `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=hmdb`, `SNAPSHOT_STORE=hurrahdb`, `SNAPSHOT_STORE=fs_db`, `SNAPSHOT_STORE=sqjson`, `SNAPSHOT_STORE=icefalldb`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=mace`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=nebari`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=rcask`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=rskey`, `SNAPSHOT_STORE=readb`, `SNAPSHOT_STORE=kv`, `SNAPSHOT_STORE=eight`, `SNAPSHOT_STORE=epoch_db`, `SNAPSHOT_STORE=etchdb`, `SNAPSHOT_STORE=fastkv`, `SNAPSHOT_STORE=rustlite`, `SNAPSHOT_STORE=rusty_leveldb`, `SNAPSHOT_STORE=canopydb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=crepedb`, `SNAPSHOT_STORE=crystal`, `SNAPSHOT_STORE=scdb`, `SNAPSHOT_STORE=skv`, `SNAPSHOT_STORE=surrealkv`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`, `SNAPSHOT_STORE=cacache`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=db_rs`, `SNAPSHOT_STORE=dharmadb`, `SNAPSHOT_STORE=dir_cache`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=saturn`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=yakv`, `SNAPSHOT_STORE=yakvdb`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=smolldb`, `SNAPSHOT_STORE=kstone`, `SNAPSHOT_STORE=roughdb`, `SNAPSHOT_STORE=raindb`, `SNAPSHOT_STORE=infusedb`, `SNAPSHOT_STORE=kafi`, `SNAPSHOT_STORE=tinkv`, `SNAPSHOT_STORE=ledger_kv`, `SNAPSHOT_STORE=blazeup`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=joydb`, `SNAPSHOT_STORE=png_db`, `SNAPSHOT_STORE=koit`, `SNAPSHOT_STORE=lite_db`, `SNAPSHOT_STORE=lmdb_rs_core`, `SNAPSHOT_STORE=log_kv`, `SNAPSHOT_STORE=loro_kv`, `SNAPSHOT_STORE=luckdb`, `SNAPSHOT_STORE=deeb`, `SNAPSHOT_STORE=lsm_engine`, `SNAPSHOT_STORE=lsm_storage_engine`, `SNAPSHOT_STORE=lsmdb`, `SNAPSHOT_STORE=lsm_tree`, `SNAPSHOT_STORE=mindb`, `SNAPSHOT_STORE=mmdb`, `SNAPSHOT_STORE=mu_db`, `SNAPSHOT_STORE=nanodb`, `SNAPSHOT_STORE=jfs`, `SNAPSHOT_STORE=simple_db`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=shorterdb`, `SNAPSHOT_STORE=celerix_store`, `SNAPSHOT_STORE=citadeldb`, `SNAPSHOT_STORE=s3`, 또는 `SNAPSHOT_STORE=managed`로 서버를 띄운 뒤 문서를 만든 다음 프로세스를 재시작해 같은 문서 ID가 hydrate되는지 확인한다. 단, `ROOM_LOCATOR != local` 또는 `ROOM_COORDINATOR=file|sqlite|managed` 같은 distributed ownership 모드에서는 startup eager hydrate 대신 ownership 확인 뒤 on-demand restore가 일어나므로, 실제 owner handoff 검증은 snapshot store와 authoritative coordination backend를 함께 맞춘 뒤 이전 owner 종료 후 새 owner의 detail/WS 진입이 최신 snapshot을 복구하는지 확인해야 한다.
