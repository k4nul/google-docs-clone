# Backend Collaborative Server

Axum, Tokio, Yrs 기반의 실시간 협업 편집 백엔드 부트스트랩 프로젝트입니다. 문서 단위 협업 서버를 빠르게 시작할 수 있도록 HTTP API, WebSocket 동기화 경계, room registry, snapshot 저장 추상화, 역할/운영 규칙을 함께 제공합니다.

현재 운영 범위는 단일 프로세스 기준입니다. 다중 프로세스 분산 전략은 문서화되어 있지만, 외부 snapshot store와 owner coordination 저장소가 준비되기 전까지는 한 `doc_id`를 하나의 프로세스만 소유해야 합니다.

## 문서 바로가기

- [Agent Rules](./docs/agent-rules.md)
- [Setup](./docs/setup.md)
- [Architecture](./docs/architecture.md)
- [API](./docs/api.md)
- [Roles](./docs/roles.md)
- [Conventions](./docs/conventions.md)
- [Checklist](./docs/checklist.md)

## 프로젝트 개요

문서 단위의 실시간 협업 서버를 Rust로 안전하게 시작할 수 있도록 최소 실행 구조를 제공합니다. 현재 단계에서는 HTTP 헬스체크, 문서 생성/조회/삭제 API, 문서별 WebSocket 진입점, in-memory room registry, 그리고 memory/file/agdb/amandine/apex_store/armdb/assystem/flash_kv/ghaladb/blockbucket/grebedb/grumpydb/graus_db/highlandcows_isam/simple_db/docdb/emdb/osmiumdb/eight/epoch_db/etchdb/fastkv/ferrumdb/rumdb/rubin/shorterdb/sqlite/heed/hightower_kv/hmdb/hurrahdb/fs_db/sqjson/bitask/bitkv_rs/bitcask_engine/blazeup/candystore/celerix_store/citadeldb/cuendillar/data_pile/jammdb/mace/janql/jasondb/jasonisnthappy/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/roughdb/raindb/infusedb/kafi/tinkv/ledger_kv/jsondb/kv/koit/lite_db/lmdb_rs_core/log_kv/mhdb/marble/loro_kv/luckdb/ipjdb/kagi/deeb/rubin/lsm_engine/lsm_storage_engine/lsmdb/lsm_tree/mindb/mmdb/mu_db/nanodb/jfs/json_store/json_db_rs/cdb64/json_mutex_db/toiletdb/dir_cache/feoxdb/s3/managed snapshot 저장 추상화를 포함합니다.

기본 빌드는 `memory`, `file`, `sqlite`, `s3`, `managed` snapshot backend만 컴파일합니다. 전체 adapter 인벤토리와 확장 회귀 테스트가 필요하면 `cargo check --features full-snapshot-stores` 또는 `cargo test --features full-snapshot-stores`를 사용합니다.

## 해결하려는 문제

협업 편집 시스템은 HTTP API, WebSocket 세션, 문서별 상태 관리, CRDT 동기화 경계를 초기에 잘 나누지 않으면 빠르게 복잡해집니다. 이 레포는 그 복잡도를 초기에 제어하기 위해 compile-safe한 기본 골격과 문서화를 함께 제공합니다.

## 핵심 기능

- `GET /api/health` 헬스체크
- `GET /api/documents` active room과 persisted snapshot을 합친 문서 목록 조회
- `POST /api/documents` 문서 생성 및 room 초기화
- `GET /api/documents/:id` 기존 문서 상세 조회
- `DELETE /api/documents/:id` 문서 및 room 제거
- `GET /ws/:doc_id` 문서별 협업 WebSocket 진입점
- 로컬 프런트엔드 개발을 단순화하기 위한 인증 없는 HTTP/WebSocket 진입점
- `DashMap` 기반 room registry와 idle room eviction
- `yrs-axum` 기반 broadcast group 연결
- `SnapshotStore` trait 및 memory/file/agdb/amandine/apex_store/armdb/assystem/flash_kv/ghaladb/blockbucket/grebedb/grumpydb/graus_db/highlandcows_isam/simple_db/docdb/emdb/osmiumdb/eight/epoch_db/etchdb/fastkv/ferrumdb/rumdb/rubin/shorterdb/sqlite/heed/hightower_kv/hmdb/hurrahdb/fs_db/sqjson/bitask/bitkv_rs/bitcask_engine/blazeup/candystore/celerix_store/citadeldb/cuendillar/data_pile/jammdb/mace/janql/jasondb/jasonisnthappy/fjall/persy/persistent_kv/native_db/nebari/nikidb/nodb/okofdb/parity_db/pickledb/rcask/microkv/redb/rskey/readb/rustlite/rustcask/rusty_leveldb/canopydb/caves/ckydb/crepedb/scdb/skv/surrealkv/sled/rustbreak/yedb/btree_store/siamesedb/structsy/abyssiniandb/aeternusdb/thunderdb/thetadb/tinybase/tinydb/dblite/dbless/db_rs/dharmadb/sanakirja/snaildb/tinykv/vsdb/yakv/saberdb/smolldb/kstone/roughdb/raindb/infusedb/kafi/tinkv/ledger_kv/jsondb/joydb/png_db/kopperdb/kv/koit/lite_db/lmdb_rs_core/log_kv/mhdb/marble/loro_kv/luckdb/ipjdb/kagi/deeb/rubin/lsm_engine/lsm_storage_engine/lsmdb/lsm_tree/mindb/mmdb/mu_db/nanodb/jfs/json_store/json_db_rs/cdb64/json_mutex_db/toiletdb/dir_cache/feoxdb/s3/managed adapter
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

기본 실행 주소는 `127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`으로 설정되어 있어 로컬 프런트엔드 개발 서버와 포트가 겹치지 않습니다. 기본 `SNAPSHOT_STORE`는 `file`이라 문서 snapshot은 `SNAPSHOT_DIR` 아래에 저장됩니다.

## 검증 흐름

```bash
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
cargo check --features full-snapshot-stores
```

- `./scripts/preflight.sh commit`는 `.git` 메타데이터 쓰기 가능 여부를 먼저 확인해 commit/stage 차단을 조기에 드러낸다.
- `./scripts/preflight.sh publish`는 여기에 `github.com` DNS 확인을 더해 push 가능성을 사전에 확인한다.
- `./scripts/preflight.sh websocket`는 socket bind가 필요한 WebSocket 검증 레인이 현재 러너에서 실행 가능한지 probe test로 확인한다.
- `./scripts/verify.sh core`는 `cargo fmt --check`, `cargo check --locked`, 그리고 socket bind가 필요 없는 테스트만 실행한다. commit/push 가능 여부와는 분리돼 있어 sandbox 환경에서도 core 검증을 막지 않는다.
- `./scripts/verify.sh websocket`는 socket bind가 필요한 WebSocket/삭제 통합 테스트만 분리 실행한다.
- 전체 snapshot adapter inventory를 다시 컴파일하거나 회귀를 돌릴 때는 `--features full-snapshot-stores`를 추가한다.
- socket-required 테스트를 새로 추가하면 `scripts/verify.sh`의 core skip 목록과 websocket lane을 함께 갱신한다.

## 역할 분담

- `A` PM / Integration: 범위 정의, 일정 관리, 프런트-백엔드 계약 조율, 통합 우선순위 결정
- `B` Frontend Editor / UI Owner: 프런트 전용 레포에서 편집기 UI를 구현하고, 이 백엔드 레포에는 API/WS 계약과 연결 검증 문서만 반영
- `C` Backend Realtime / API Owner: HTTP API, room registry, WebSocket 협업, CRDT 서버 구조 유지
- `D` QA / Docs / DevOps Owner: 테스트 실행, 문서 최신화, 실행 절차 검증, 릴리스/운영 준비

## 협업 규칙

- 커밋 메시지는 `type(scope): subject` 형식을 사용하고, `type`은 `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`, `rename`, `remove`만 사용한다.
- `scope`는 `api`, `sync`, `yrs`, `auth`, `db`, `websocket`, `storage`, `config`, `docs`, `repo` 중에서 변경 의미가 드러나도록 고른다.
- `subject`는 현재형, 소문자 시작, 마침표 없음, 변경 내용을 직접 설명하는 문장 조각으로 작성한다.
- 한 커밋에는 한 가지 목적만 담고, 리팩토링과 동작 변경을 섞지 않는다.
- API, WebSocket, 환경변수, 스키마가 바뀌면 `README.md`와 관련 `/docs` 문서를 같은 작업 안에서 함께 갱신한다.
- 이 백엔드 레포에는 개발 역할 브랜치만 유지하고, 현재 협업 브랜치는 `backend-realtime-api`와 `qa-docs-devops`다.
- 작업 브랜치는 `main`에서 분기하고, 직접 `main`에 push하지 않고 PR로 병합한다.
- PR을 올리기 전에는 가능하면 `cargo fmt --check`, `cargo check`, `cargo test` 결과를 남기고 최신 `main` 기준 충돌을 정리한다.

## API/WS 개요

- HTTP base path: `/api`
- Health: `GET /api/health`
- Documents: `GET /api/documents`, `POST /api/documents`, `GET /api/documents/:id`, `DELETE /api/documents/:id`
- Collaboration WebSocket: `GET /ws/:doc_id`

현재 문서 API와 협업 WebSocket은 `Authorization` 헤더 없이 동작합니다. `POST /api/documents` 응답에는 저장소 호환을 위한 문서 전용 `access_token`이 계속 포함되지만, 클라이언트가 이후 요청에 이 값을 보낼 필요는 없습니다. 존재하지 않는 문서 ID로 상세 조회나 WebSocket 연결을 시도하면 `404`를 반환합니다. 활성 협업 WebSocket 세션이 남아 있는 문서를 삭제하려 하면 `409 conflict`를 반환합니다. WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 합니다. active room이 없으면 snapshot store에서 room을 복구하고, Yrs document update가 commit될 때마다 최신 snapshot을 저장합니다. 마지막 WebSocket 세션이 종료되면 한 번 더 snapshot을 저장한 뒤 idle room을 메모리에서 제거합니다.

non-local owner 때문에 `409 conflict`가 반환될 때는 기존 JSON body와 함께 ingress/proxy 레이어가 바로 사용할 수 있도록 `x-collab-owner-node-id` 헤더가 추가됩니다. `owner.base_url`이 있으면 canonical owner origin을 담은 `x-collab-owner-base-url`, 현재 요청 path/query를 owner origin에 붙인 `x-collab-redirect-location`, 그리고 표준 `Location` 헤더도 함께 실립니다.

## WebSocket Binary Format

프런트엔드는 문서 diff를 JSON이나 multipart 파일로 보내지 않고, `/ws/:doc_id` WebSocket binary frame에 Yjs/Yrs sync protocol 메시지를 담아 보낸다. 이 서버는 `yrs::sync::Message::encode_v1()`와 호환되는 v1 binary payload를 받는다.

```text
WebSocket binary frame
`-- Yrs Message v1
    |-- message type
    `-- payload
```

Top-level message type:

| 값 | 의미 |
| --- | --- |
| `0` | `Sync` |
| `1` | `Awareness` |
| `2` | `Auth` |
| `3` | `AwarenessQuery` |

`Sync` message 내부 type:

| 값 | 의미 | 사용 시점 |
| --- | --- | --- |
| `0` | `SyncStep1` | 연결 직후 클라이언트 state vector 전송 |
| `1` | `SyncStep2` | 상대 state vector 기준으로 만든 update 응답 |
| `2` | `Update` | 편집 중 발생한 Yjs document update diff 전송 |

기본 흐름:

1. 클라이언트가 `/ws/:doc_id`에 WebSocket으로 연결한다.
2. 클라이언트가 binary `Sync(SyncStep1(stateVector))`를 보낸다.
3. 서버가 binary `Sync(SyncStep2(update))`를 반환한다.
4. 클라이언트가 받은 update를 로컬 `Y.Doc`에 적용한다.
5. 편집이 발생하면 클라이언트가 binary `Sync(Update(update))`를 보낸다.
6. 서버는 같은 `doc_id` room의 다른 클라이언트에게 binary `Sync(Update(update))`를 broadcast한다.

브라우저 프런트에서 직접 인코딩할 때의 예시는 아래와 같다. 일반적으로는 같은 포맷을 처리하는 Yjs provider를 사용하는 편이 안전하다.

```ts
import * as Y from "yjs";
import * as encoding from "lib0/encoding";
import * as decoding from "lib0/decoding";
import * as syncProtocol from "y-protocols/sync";
import * as awarenessProtocol from "y-protocols/awareness";

const MSG_SYNC = 0;
const MSG_AWARENESS = 1;

const doc = new Y.Doc();
const awareness = new awarenessProtocol.Awareness(doc);
const ws = new WebSocket(`ws://localhost:3000/ws/${docId}`);
ws.binaryType = "arraybuffer";

ws.onopen = () => {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, MSG_SYNC);
  syncProtocol.writeSyncStep1(encoder, doc);
  ws.send(encoding.toUint8Array(encoder));
};

doc.on("update", (update: Uint8Array) => {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, MSG_SYNC);
  syncProtocol.writeUpdate(encoder, update);
  ws.send(encoding.toUint8Array(encoder));
});

ws.onmessage = (event) => {
  const data = new Uint8Array(event.data);
  const decoder = decoding.createDecoder(data);
  const messageType = decoding.readVarUint(decoder);

  if (messageType === MSG_SYNC) {
    const reply = encoding.createEncoder();
    encoding.writeVarUint(reply, MSG_SYNC);
    syncProtocol.readSyncMessage(decoder, reply, doc, ws);

    const replyBytes = encoding.toUint8Array(reply);
    if (replyBytes.length > 1) {
      ws.send(replyBytes);
    }
  }

  if (messageType === MSG_AWARENESS) {
    const update = decoding.readVarUint8Array(decoder);
    awarenessProtocol.applyAwarenessUpdate(awareness, update, "remote");
  }
};
```

Awareness는 JSON을 WebSocket text frame으로 직접 보내지 않는다. 프런트가 아래 구조를 Yjs awareness local state로 설정하면, provider 또는 `y-protocols/awareness`가 `Awareness` binary message로 인코딩해 전송한다. 서버는 이 JSON shape를 검증한 뒤 room awareness state에 반영한다.

브라우저 기본 `WebSocket` API는 임의의 `Authorization` 헤더를 직접 설정할 수 없으므로, 현재 로컬 개발 계약에서는 `/ws/:doc_id` 연결에 인증 헤더를 요구하지 않는다. WebSocket 연결에는 `FRONTEND_ORIGIN`과 일치하는 `Origin` 헤더만 필요하다.

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
- `API_TOKEN`: 현재 로컬 개발용 HTTP/WS 경로에서는 검증하지 않는 legacy 관리 토큰 설정
- `SNAPSHOT_STORE`: `memory`, `file`, `agdb`, `amandine`, `append_log`, `apex_store`, `armdb`, `assystem`, `colon_db`, `flash_kv`, `ghaladb`, `blockbucket`, `grebedb`, `grumpydb`, `graus_db`, `highlandcows_isam`, `simple_db`, `docdb`, `emdb`, `osmiumdb`, `eight`, `epoch_db`, `etchdb`, `fastkv`, `ferrumdb`, `rumdb`, `rubin`, `shorterdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `hurrahdb`, `fs_db`, `sqjson`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `citadeldb`, `cuendillar`, `data_pile`, `datastack`, `jammdb`, `mace`, `janql`, `jasondb`, `jasonisnthappy`, `jfs`, `json_store`, `json_db_rs`, `cdb64`, `json_mutex_db`, `toiletdb`, `feoxdb`, `jsondb`, `kopperdb`, `kv`, `koit`, `lite_db`, `lmdb_rs_core`, `log_kv`, `append_kv`, `mhdb`, `marble`, `loro_kv`, `luckdb`, `ipjdb`, `kagi`, `deeb`, `lsm_engine`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `mu_db`, `nanodb`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nikidb`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `rcask`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `canopydb`, `caves`, `ckydb`, `crepedb`, `crystal`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `rustcask`, `rusty_leveldb`, `yedb`, `btree_store`, `cacache`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `dir_cache`, `sanakirja`, `saturn`, `snaildb`, `tinykv`, `vsdb`, `yakv`, `yakvdb`, `saberdb`, `smolldb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `joydb`, `png_db`, `s3`, 또는 `managed`
- 기본 feature 없는 빌드에서 바로 쓸 수 있는 값은 `memory`, `file`, `sqlite`, `s3`, `managed`다. 나머지 backend는 `--features full-snapshot-stores`를 켜야 registry와 adapter가 함께 활성화된다.
- `SNAPSHOT_STORE=append_kv`: append_kv append-only 단일 파일 store도 지원한다.
- `SNAPSHOT_STORE=append_log`: append-log append-only 단일 파일 event log store도 지원한다.
- `SNAPSHOT_STORE=etchdb`: EtchDB WAL-backed 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=fastkv`: FastKV compressed binary dump 파일 store도 지원한다.
- `SNAPSHOT_STORE=ferrumdb`: FerrumDB append-only log 파일 store도 지원한다.
- `SNAPSHOT_STORE=cdb64`: cdb64 single-file key-value store도 지원한다.
- `SNAPSHOT_STORE=kagi`: kagi whole-file bincode hashmap store도 지원한다.
- `SNAPSHOT_STORE=armdb`: repository-local armdb shim 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=mindb`: repository-local mindb shim 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=mmdb`: repository-local mmdb shim 디렉터리 store도 지원한다.
- `SNAPSHOT_STORE=mu_db`: muDB data/index 파일 쌍 기반 key-value store도 지원한다.
- `SNAPSHOT_STORE=nanodb`: NanoDB single-file JSON store도 지원한다.
- `SNAPSHOT_DIR`: `SNAPSHOT_STORE=file`일 때 snapshot JSON 파일을 저장할 디렉터리
- `SNAPSHOT_AGDB_PATH`: `SNAPSHOT_STORE=agdb`일 때 snapshot agdb 단일 파일 경로
- `SNAPSHOT_AMANDINE_PATH`: `SNAPSHOT_STORE=amandine`일 때 snapshot Amandine 디렉터리 경로
- `SNAPSHOT_APEX_STORE_PATH`: `SNAPSHOT_STORE=apex_store`일 때 snapshot apex_store shim 디렉터리 경로. 실제 payload는 `store.json`에 저장된다
- `SNAPSHOT_ARMDB_PATH`: `SNAPSHOT_STORE=armdb`일 때 snapshot armdb shim 디렉터리 경로. 실제 payload는 `store.json`에 저장된다
- `SNAPSHOT_ASSYSTEM_PATH`: `SNAPSHOT_STORE=assystem`일 때 snapshot assystem 단일 파일 경로
- `SNAPSHOT_COLON_DB_PATH`: `SNAPSHOT_STORE=colon_db`일 때 snapshot colon_db 단일 파일 경로
- `SNAPSHOT_FLASH_KV_PATH`: `SNAPSHOT_STORE=flash_kv`일 때 snapshot flash-kv 디렉터리 경로
- `SNAPSHOT_GHALADB_PATH`: `SNAPSHOT_STORE=ghaladb`일 때 snapshot GhalaDB LSM value-log 디렉터리 경로
- `SNAPSHOT_BLOCKBUCKET_PATH`: `SNAPSHOT_STORE=blockbucket`일 때 snapshot blockbucket 단일 파일 경로
- `SNAPSHOT_GREBEDB_PATH`: `SNAPSHOT_STORE=grebedb`일 때 snapshot grebedb 디렉터리 경로
- `SNAPSHOT_GRUMPYDB_PATH`: `SNAPSHOT_STORE=grumpydb`일 때 snapshot grumpydb 디렉터리 경로
- `SNAPSHOT_GRAUS_DB_PATH`: `SNAPSHOT_STORE=graus_db`일 때 snapshot GrausDb 로그 디렉터리 경로
- `SNAPSHOT_HIGHLANDCOWS_ISAM_PATH`: `SNAPSHOT_STORE=highlandcows_isam`일 때 snapshot highlandcows-isam path prefix. 실제 저장 파일은 `<path>.idb`, `<path>.idx`
- `SNAPSHOT_SIMPLE_DB_PATH`: `SNAPSHOT_STORE=simple_db`일 때 snapshot simple_db 단일 파일 경로
- `SNAPSHOT_DOCDB_PATH`: `SNAPSHOT_STORE=docdb`일 때 snapshot docdb JSON 파일 경로
- `SNAPSHOT_EMDB_PATH`: `SNAPSHOT_STORE=emdb`일 때 snapshot emdb DB 파일 경로. adapter는 `EmdbBuilder::prefer_v4(true)`와 explicit flush 경계로 v0.7 engine을 고정한다.
- `SNAPSHOT_OSMIUMDB_PATH`: `SNAPSHOT_STORE=osmiumdb`일 때 snapshot OsmiumDB 디렉터리 경로. adapter는 save/delete마다 `flush()` 뒤 `checkpoint()`를 호출해 WAL replay와 map snapshot reopen 경계를 함께 고정한다.
- `SNAPSHOT_EIGHT_PATH`: `SNAPSHOT_STORE=eight`일 때 snapshot eight 디렉터리 경로
- `SNAPSHOT_EPOCH_DB_PATH`: `SNAPSHOT_STORE=epoch_db`일 때 snapshot epoch-db 디렉터리 경로
- `SNAPSHOT_ETCHDB_PATH`: `SNAPSHOT_STORE=etchdb`일 때 snapshot EtchDB WAL 디렉터리 경로
- `SNAPSHOT_FASTKV_PATH`: `SNAPSHOT_STORE=fastkv`일 때 snapshot FastKV compressed binary dump 파일 경로
- `SNAPSHOT_FERRUMDB_PATH`: `SNAPSHOT_STORE=ferrumdb`일 때 snapshot FerrumDB append-only log 파일 경로
- `SNAPSHOT_RUMDB_PATH`: `SNAPSHOT_STORE=rumdb`일 때 snapshot rumdb append-only log 디렉터리 경로
- `SNAPSHOT_SHORTERDB_PATH`: `SNAPSHOT_STORE=shorterdb`일 때 snapshot shorterdb 디렉터리 경로
- `SNAPSHOT_SQLITE_PATH`: `SNAPSHOT_STORE=sqlite`일 때 snapshot repository-local sqlite shim 파일 경로. adapter는 같은 경로의 logical `snapshots` table JSON payload와 `<path>.lock` sidecar lock을 사용한다
- `SNAPSHOT_HEED_PATH`: `SNAPSHOT_STORE=heed`일 때 snapshot heed DB 디렉터리 경로
- `SNAPSHOT_HIGHTOWER_KV_PATH`: `SNAPSHOT_STORE=hightower_kv`일 때 snapshot hightower-kv 데이터 디렉터리 경로
- `SNAPSHOT_HMDB_PATH`: `SNAPSHOT_STORE=hmdb`일 때 snapshot hmdb append-only 로그 디렉터리 경로
- `SNAPSHOT_HURRAHDB_PATH`: `SNAPSHOT_STORE=hurrahdb`일 때 snapshot HurrahDB append-only 파일 경로
- `SNAPSHOT_FS_DB_PATH`: `SNAPSHOT_STORE=fs_db`일 때 snapshot fs-db key-per-file 디렉터리 경로
- `SNAPSHOT_SQJSON_PATH`: `SNAPSHOT_STORE=sqjson`일 때 snapshot sqjson single-file JSON DB 경로
- `SNAPSHOT_ICEFALLDB_PATH`: `SNAPSHOT_STORE=icefalldb`일 때 snapshot icefalldb 로그 디렉터리 경로
- `SNAPSHOT_BITASK_PATH`: `SNAPSHOT_STORE=bitask`일 때 snapshot bitask append-only log 디렉터리 경로
- `SNAPSHOT_BITKV_RS_PATH`: `SNAPSHOT_STORE=bitkv_rs`일 때 snapshot bitkv-rs append-only log 디렉터리 경로
- `SNAPSHOT_BITCASK_ENGINE_PATH`: `SNAPSHOT_STORE=bitcask_engine`일 때 snapshot bitcask-engine-rs append-only log 디렉터리 경로
- `SNAPSHOT_BLAZEUP_PATH`: `SNAPSHOT_STORE=blazeup`일 때 snapshot blazeup/kv sled 디렉터리 경로
- `SNAPSHOT_CANDYSTORE_PATH`: `SNAPSHOT_STORE=candystore`일 때 snapshot candystore 디렉터리 경로
- `SNAPSHOT_CELERIX_STORE_PATH`: `SNAPSHOT_STORE=celerix_store`일 때 snapshot Celerix Store JSON persistence 디렉터리 경로
- `SNAPSHOT_CITADELDB_PATH`: `SNAPSHOT_STORE=citadeldb`일 때 encrypted CitadelDB snapshot DB 파일 경로
- `SNAPSHOT_CITADELDB_PASSPHRASE`: `SNAPSHOT_STORE=citadeldb`일 때 DB key file을 여는 passphrase
- `SNAPSHOT_CUENDILLAR_PATH`: `SNAPSHOT_STORE=cuendillar`일 때 snapshot cuendillar 루트 디렉터리 경로. 내부에 `wal/`, `sstable/` 디렉터리가 함께 생성된다
- `SNAPSHOT_DATA_PILE_PATH`: `SNAPSHOT_STORE=data_pile`일 때 snapshot data-pile append-only 디렉터리 경로
- `SNAPSHOT_DATASTACK_PATH`: `SNAPSHOT_STORE=datastack`일 때 snapshot DataStack redb 파일 경로
- `SNAPSHOT_JAMMDB_PATH`: `SNAPSHOT_STORE=jammdb`일 때 snapshot jammdb 파일 경로
- `SNAPSHOT_MACE_PATH`: `SNAPSHOT_STORE=mace`일 때 snapshot Mace 디렉터리 경로
- `SNAPSHOT_JANQL_PATH`: `SNAPSHOT_STORE=janql`일 때 snapshot janql WAL/SSTable 디렉터리 경로
- `SNAPSHOT_JASONDB_PATH`: `SNAPSHOT_STORE=jasondb`일 때 snapshot JasonDB append-only 파일 경로
- `SNAPSHOT_JASONISNTHAPPY_PATH`: `SNAPSHOT_STORE=jasonisnthappy`일 때 snapshot jasonisnthappy 단일 DB 파일 경로
- `SNAPSHOT_FJALL_PATH`: `SNAPSHOT_STORE=fjall`일 때 snapshot fjall DB 디렉터리 경로
- `SNAPSHOT_PERSY_PATH`: `SNAPSHOT_STORE=persy`일 때 snapshot persy 파일 경로
- `SNAPSHOT_PERSISTENT_KV_PATH`: `SNAPSHOT_STORE=persistent_kv`일 때 snapshot persistent-kv 디렉터리 경로
- `SNAPSHOT_NATIVE_DB_PATH`: `SNAPSHOT_STORE=native_db`일 때 snapshot native_db 파일 경로
- `SNAPSHOT_NEBARI_PATH`: `SNAPSHOT_STORE=nebari`일 때 snapshot nebari shim 디렉터리 경로. 실제 payload는 `store.json`에 저장된다
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
- `SNAPSHOT_CRYSTAL_PATH`: `SNAPSHOT_STORE=crystal`일 때 snapshot crystal key-per-file 디렉터리 경로
- `SNAPSHOT_SCDB_PATH`: `SNAPSHOT_STORE=scdb`일 때 snapshot scdb 디렉터리 경로
- `SNAPSHOT_SKV_PATH`: `SNAPSHOT_STORE=skv`일 때 snapshot skv base path. 실제 저장 파일은 `<path>.data`, `<path>.index`
- `SNAPSHOT_SURREALKV_PATH`: `SNAPSHOT_STORE=surrealkv`일 때 snapshot surrealkv B+tree 단일 파일 경로
- `SNAPSHOT_SLED_PATH`: `SNAPSHOT_STORE=sled`일 때 snapshot sled DB 디렉터리 경로
- `SNAPSHOT_RUSTBREAK_PATH`: `SNAPSHOT_STORE=rustbreak`일 때 snapshot rustbreak 단일 파일 경로
- `SNAPSHOT_YEDB_PATH`: `SNAPSHOT_STORE=yedb`일 때 snapshot yedb DB 디렉터리 경로
- `SNAPSHOT_BTREE_STORE_PATH`: `SNAPSHOT_STORE=btree_store`일 때 snapshot btree-store 단일 파일 경로
- `SNAPSHOT_CACACHE_PATH`: `SNAPSHOT_STORE=cacache`일 때 snapshot cacache 디렉터리 경로
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
- `SNAPSHOT_SATURN_PATH`: `SNAPSHOT_STORE=saturn`일 때 snapshot SaturnDB WAL 파일 경로
- `SNAPSHOT_SNAILDB_PATH`: `SNAPSHOT_STORE=snaildb`일 때 snapshot snaildb 디렉터리 경로
- `SNAPSHOT_TINYKV_PATH`: `SNAPSHOT_STORE=tinykv`일 때 snapshot tinykv JSON 파일 경로
- `SNAPSHOT_VSDB_PATH`: `SNAPSHOT_STORE=vsdb`일 때 snapshot vsdb handle metadata 디렉터리 경로. `store.meta.json`이 생성되며, 실제 payload map file은 `VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb` 아래 `maps/<instance_id>.json`에 저장된다
- `SNAPSHOT_YAKV_PATH`: `SNAPSHOT_STORE=yakv`일 때 snapshot yakv 단일 파일 경로
- `SNAPSHOT_YAKVDB_PATH`: `SNAPSHOT_STORE=yakvdb`일 때 snapshot yakvdb 단일 파일 경로
- `SNAPSHOT_SABERDB_PATH`: `SNAPSHOT_STORE=saberdb`일 때 snapshot saberdb JSON 파일 경로
- `SNAPSHOT_SMOLLDB_PATH`: `SNAPSHOT_STORE=smolldb`일 때 snapshot SmollDB compressed 단일 파일 경로
- `SNAPSHOT_KSTONE_PATH`: `SNAPSHOT_STORE=kstone`일 때 snapshot repository-local kstone shim 디렉터리 경로. adapter는 `wal.log` append-only log를 replay해 keyspace를 복구한다
- `SNAPSHOT_ROUGHDB_PATH`: `SNAPSHOT_STORE=roughdb`일 때 snapshot roughdb shim 디렉터리 경로. 실제 payload는 `store.json`에 저장된다
- `SNAPSHOT_RAINDB_PATH`: `SNAPSHOT_STORE=raindb`일 때 snapshot RainDB WAL/SSTable 디렉터리 경로
- `SNAPSHOT_INFUSEDB_PATH`: `SNAPSHOT_STORE=infusedb`일 때 snapshot InfuseDB 단일 파일 경로
- `SNAPSHOT_KAFI_PATH`: `SNAPSHOT_STORE=kafi`일 때 snapshot kafi 단일 파일 경로
- `SNAPSHOT_TINKV_PATH`: `SNAPSHOT_STORE=tinkv`일 때 snapshot tinkv 디렉터리 경로
- `SNAPSHOT_LEDGER_KV_PATH`: `SNAPSHOT_STORE=ledger_kv`일 때 snapshot ledger-kv append-only ledger 디렉터리 경로
- `SNAPSHOT_FEOXDB_PATH`: `SNAPSHOT_STORE=feoxdb`일 때 snapshot FeOxDB 단일 파일 경로
- `SNAPSHOT_JSONDB_PATH`: `SNAPSHOT_STORE=jsondb`일 때 snapshot jsondb JSON 파일 경로
- `SNAPSHOT_JOYDB_PATH`: `SNAPSHOT_STORE=joydb`일 때 snapshot Joydb JSON 파일 경로
- `SNAPSHOT_PNG_DB_PATH`: `SNAPSHOT_STORE=png_db`일 때 snapshot png-db PNG 파일 경로
- `SNAPSHOT_KOPPERDB_PATH`: `SNAPSHOT_STORE=kopperdb`일 때 snapshot kopperdb 세그먼트 디렉터리 경로
- `SNAPSHOT_KV_PATH`: `SNAPSHOT_STORE=kv`일 때 snapshot kv sled 디렉터리 경로
- `SNAPSHOT_KOIT_PATH`: `SNAPSHOT_STORE=koit`일 때 snapshot koit JSON 파일 경로
- `SNAPSHOT_LITE_DB_PATH`: `SNAPSHOT_STORE=lite_db`일 때 snapshot LiteDb 디렉터리 경로
- `SNAPSHOT_LOG_KV_PATH`: `SNAPSHOT_STORE=log_kv`일 때 snapshot append-only 단일 파일 경로
- `SNAPSHOT_APPEND_KV_PATH`: `SNAPSHOT_STORE=append_kv`일 때 snapshot append_kv append-only 단일 파일 경로
- `SNAPSHOT_APPEND_LOG_PATH`: `SNAPSHOT_STORE=append_log`일 때 snapshot append-log append-only 단일 파일 경로
- `SNAPSHOT_MHDB_PATH`: `SNAPSHOT_STORE=mhdb`일 때 snapshot MHdb DB path prefix. 실제 저장 파일은 `<path>.pag`, `<path>.dir`
- `SNAPSHOT_LORO_KV_PATH`: `SNAPSHOT_STORE=loro_kv`일 때 snapshot loro_kv single-file export 경로
- `SNAPSHOT_LUCKDB_PATH`: `SNAPSHOT_STORE=luckdb`일 때 snapshot LuckDB JSON document 파일 경로
- `SNAPSHOT_IPJDB_PATH`: `SNAPSHOT_STORE=ipjdb`일 때 snapshot ipjdb collection 디렉터리 경로
- `SNAPSHOT_KAGI_PATH`: `SNAPSHOT_STORE=kagi`일 때 snapshot kagi whole-file store 경로
- `SNAPSHOT_DEEB_PATH`: `SNAPSHOT_STORE=deeb`일 때 snapshot Deeb JSON database 파일 경로
- `SNAPSHOT_RUBIN_PATH`: `SNAPSHOT_STORE=rubin`일 때 snapshot Rubin JSON 파일 경로
- `SNAPSHOT_LSM_ENGINE_PATH`: `SNAPSHOT_STORE=lsm_engine`일 때 snapshot lsm_engine WAL 파일 경로
- `SNAPSHOT_LSM_STORAGE_ENGINE_PATH`: `SNAPSHOT_STORE=lsm_storage_engine`일 때 snapshot lsm_storage_engine WAL/SSTable 디렉터리 경로
- `SNAPSHOT_LSMDB_PATH`: `SNAPSHOT_STORE=lsmdb`일 때 snapshot lsmdb WAL/SSTable 디렉터리 경로
- `SNAPSHOT_LSM_TREE_PATH`: `SNAPSHOT_STORE=lsm_tree`일 때 snapshot lsm-tree primitive 디렉터리 경로
- `SNAPSHOT_MINDB_PATH`: `SNAPSHOT_STORE=mindb`일 때 snapshot mindb shim 디렉터리 경로. 실제 state는 `store.json`, replay log는 `wal.log`, manifest는 `manifest.json`에 저장된다
- `SNAPSHOT_MMDB_PATH`: `SNAPSHOT_STORE=mmdb`일 때 snapshot mmdb shim 디렉터리 경로. 실제 payload는 `store.json`에 저장된다
- `SNAPSHOT_MU_DB_PATH`: `SNAPSHOT_STORE=mu_db`일 때 snapshot muDB data 파일 경로. 같은 디렉터리에 `index_<file_name>` index 파일도 함께 생성된다.
- `SNAPSHOT_NANODB_PATH`: `SNAPSHOT_STORE=nanodb`일 때 snapshot NanoDB single JSON 파일 경로
- `SNAPSHOT_JFS_PATH`: `SNAPSHOT_STORE=jfs`일 때 snapshot jfs single-file JSON catalog 경로
- `SNAPSHOT_JSON_STORE_PATH`: `SNAPSHOT_STORE=json_store`일 때 snapshot json_store append-only JSON line catalog 경로
- `SNAPSHOT_JSON_DB_RS_PATH`: `SNAPSHOT_STORE=json_db_rs`일 때 snapshot json_db_rs JSON event log 파일 경로
- `SNAPSHOT_CDB64_PATH`: `SNAPSHOT_STORE=cdb64`일 때 snapshot cdb64 single-file CDB 경로
- `SNAPSHOT_JSON_MUTEX_DB_PATH`: `SNAPSHOT_STORE=json_mutex_db`일 때 snapshot json-mutex-db JSON 파일 경로
- `SNAPSHOT_TOILETDB_PATH`: `SNAPSHOT_STORE=toiletdb`일 때 snapshot ToiletDB JSON 파일 경로
- `SNAPSHOT_DIR_CACHE_PATH`: `SNAPSHOT_STORE=dir_cache`일 때 snapshot dir-cache 디렉터리 경로
- `SNAPSHOT_MARBLE_PATH`: `SNAPSHOT_STORE=marble`일 때 snapshot Marble object store 디렉터리 경로
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
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 파일을 저장하는 디렉터리이며, `ROOM_LOCATOR=file`은 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_SQLITE_PATH`: `ROOM_COORDINATOR=sqlite`일 때 lease row를 저장하는 repository-local sqlite shim 파일 경로이며, `ROOM_LOCATOR=sqlite`는 같은 파일과 `<path>.lock` sidecar를 읽는다
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
- 기본 file-backed snapshot store와 상단 `SNAPSHOT_STORE` 항목에 나열된 모든 로컬/embedded backend, S3-compatible object storage, external managed snapshot store 지원
- config-driven room locator local/static/file/sqlite/managed 모드와 room coordinator dry-run logging/file/sqlite/managed state 모드 지원

## 비범위

- 데이터베이스 연동
- 문서 수정용 REST API
- 추가 vendor-specific database durability backend

현재 기본값은 여전히 단일 프로세스다. 다만 `SNAPSHOT_STORE=sqlite`와 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite`를 같은 shared sqlite shim 파일 경로에 맞추면, lock-capable storage 위에서 `<path>.lock` sidecar serialize 경계로 lease compare-and-swap과 snapshot 내구성을 함께 가져갈 수 있다. 그 외 상단 `SNAPSHOT_STORE` 항목의 embedded/local durability backend는 같은 `SnapshotStore` 경계를 통해 로컬 durable restart 복구를 제공한다. `SNAPSHOT_STORE=celerix_store`는 `SNAPSHOT_CELERIX_STORE_PATH/snapshots.json`의 Celerix Store persona/app map에 snapshot JSON value를 저장해 startup hydrate와 on-demand restore를 유지한다. `SNAPSHOT_STORE=s3`는 object key 단위 durability를 제공하고, `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 external lease service에 연결하고 `SNAPSHOT_STORE=managed`를 external snapshot service에 연결하면 ownership coordination plane과 snapshot durability plane을 shared sqlite shim 밖으로도 분리할 수 있다. 현재 저장소는 managed coordination + managed snapshot durability 조합까지 실제 multi-host handoff 회귀 테스트로 검증한다.

현재 `blocked` 상태는 실행 환경 차원의 commit/push/test 제한을 별도 관리하는 정도로 축소됐다. 반면 상단 `SNAPSHOT_STORE` 항목에서 `memory`를 제외한 durability backend와 managed-managed owner handoff rehearsal은 이제 회귀 테스트로 검증됐다.

## Embedded Snapshot Store Selection Guide

다음 기준은 상단 `SNAPSHOT_STORE` 항목에서 `memory`, `sqlite`, `s3`, `managed`를 제외한 embedded/local durability backend 중 어떤 것을 운영 기본값으로 둘지 고를 때 사용하는 README 요약판이다. backend별 상세 운영 매트릭스와 기준선 목록은 canonical reference로 [`docs/setup.md`](docs/setup.md)의 같은 섹션만 유지한다.

| 운영 질문 | 빠른 판단 |
| --- | --- |
| 실제 multi-node owner handoff까지 같은 저장소에서 끝내야 하는가 | `sqlite` 또는 `managed`를 쓴다. embedded backend는 snapshot durability만 제공하고 authoritative lease CAS는 제공하지 않는다. |
| 단일 노드 재시작 복구가 목표인가 | `docs/setup.md`의 backend matrix에서 저장 단위가 단일 파일, 단일 path, 또는 단일 엔진 디렉터리인 후보를 먼저 본다. backup/restore 절차가 가장 단순하다. |
| 운영자가 payload를 직접 열어보며 수동 복구해야 하는가 | `docs/setup.md` matrix에서 운영자 payload 가시성이 높은 text-oriented 후보를 우선 본다. 대가로 whole-file parse나 catalog scan 비용은 더 보수적으로 본다. |
| corrupt entry skip 같은 손상 격리 특성이 중요한가 | `docs/setup.md` matrix에서 손상/복구 주의점이 entry skip 또는 explicit catalog key 중심으로 정리된 후보를 우선 본다. whole-file rewrite/store 전체 역직렬화 의존 후보는 기본값으로 둘 때 더 주의한다. |
| pure-Rust/no-bindgen/no-native-conflict 제약을 유지해야 하는가 | `docs/setup.md` matrix의 제약 메모에서 같은 기준선을 유지하는 후보만 필터링한다. 새 backend screening도 이 기준을 먼저 통과해야 한다. |

README에는 위 질문만 남기고, backend별 저장 단위와 손상/복구 메모, pure-Rust 기준선 열거는 `docs/setup.md`의 canonical matrix를 직접 참조한다. 새 snapshot backend를 추가하거나 분류를 바꿀 때도 README의 수동 backend 열거를 다시 맞추지 않고 `docs/setup.md` 한 곳만 갱신하면 된다.

`ROOM_LOCATOR=static`은 외부 coordinator를 대체하지 않는다. 대신 운영자가 문서별 owner 힌트를 선언해 현재 노드 비소유 문서를 조기에 거절하고, 응답 JSON의 `owner.node_id` / optional `owner.base_url` 및 대응 헤더로 upstream 라우팅 결정을 돕는 용도다. 힌트에 없는 문서는 현재 노드 소유로 간주한다.

`ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active room lease state를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `FileRoomCoordinator`가 같은 디렉터리에 남긴 state를 소비하는 best-effort resolver이며, `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 upstream 라우팅 결정을 도울 수 있다. stale owner 판단은 file mtime이 아니라 persisted `expires_at`만 기준으로 한다.

`ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 repository-local sqlite shim 파일 안 logical `room_leases` row를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `SqliteRoomCoordinator`가 같은 파일에 기록한 lease를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

`ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에서 `GET /v1/leases/:doc_id`를 조회해 현재 노드 비소유 문서를 거절한다. 이 모드는 `ManagedRoomCoordinator`가 같은 service에 기록한 canonical lease record를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

## 향후 확장 방향

- provider awareness payload 연동
- 외부 저장소 adapter 추가
- 별도 frontend 레포와 provider / editor 연동 계약 고도화
- 추가 vendor-specific database durability backend

## Snapshot Restore / Eviction Policy

- 문서 생성 시 초기 snapshot을 저장하고 active room을 메모리에 등록한다.
- `GET /api/documents`는 active room이 없어도 snapshot store에 남아 있는 문서를 카탈로그로 반환한다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 먼저 `RoomLocator`로 현재 노드 ownership을 확인한 뒤, active room이 없으면 snapshot store에서 room을 on-demand로 복구한다.
- Yrs document update가 commit될 때마다 room의 full-state update 바이너리와 문서 metadata를 snapshot store에 저장한다.
- WebSocket 세션이 종료될 때마다 room의 active session 수를 감소시키고, 마지막 세션이 닫히면 최신 snapshot을 한 번 더 저장한 뒤 room을 메모리에서 제거한다.
- 문서가 삭제된 경우에는 snapshot과 active room을 함께 제거한다. 활성 WebSocket 세션이 남아 있으면 삭제를 거절하고 `409 conflict`를 반환한다.
- `SNAPSHOT_STORE=file`일 때 손상된 snapshot 파일은 startup hydrate와 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛴다. 해당 문서를 직접 복구하려고 로드하면 여전히 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=file` 저장은 같은 디렉터리의 임시 파일 작성 후 `rename`으로 마무리해, 저장 도중 프로세스가 중단돼도 마지막 정상 snapshot을 바로 덮어쓰지 않도록 한다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되고, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 정리한다.
- `SNAPSHOT_STORE=file`이면 snapshot과 문서 토큰이 `SNAPSHOT_DIR/<doc_id>.json`에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 해당 디렉터리에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=agdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_AGDB_PATH` agdb 단일 파일의 `snapshot:<doc_id>` alias node에 JSON payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 agdb alias catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=amandine`이면 snapshot과 문서 토큰이 `SNAPSHOT_AMANDINE_PATH/snapshots.json` Amandine collection record에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=apex_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_APEX_STORE_PATH/store.json` repository-local apex_store shim map에 `snapshot:<doc_id>` payload와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=assystem`이면 snapshot과 문서 토큰이 `SNAPSHOT_ASSYSTEM_PATH` 단일 assystem 파일의 `doc_id -> persisted snapshot JSON bytes` entry로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 file-backed key list에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=colon_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_COLON_DB_PATH` 단일 colon_db 파일의 `doc_id -> base64(persisted snapshot JSON)` row로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 file-backed row catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=grebedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_GREBEDB_PATH` 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. payload와 catalog는 같은 `flush()` 경계에서 함께 반영돼 기본 local ownership 모드에서는 앱 시작 시 grebedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=grumpydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_GRUMPYDB_PATH` 디렉터리의 GrumpyDB UUID key와 bytes payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 full range scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=graus_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_GRAUS_DB_PATH` 디렉터리의 GrausDb append-only log store에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay catalog로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=sqlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_SQLITE_PATH` repository-local sqlite shim 파일의 logical `snapshots` table에 저장된다. adapter는 `<path>.lock` sidecar lock 아래 whole-file JSON rewrite로 upsert/delete/list를 직렬화한다. 기본 local ownership 모드에서는 앱 시작 시 catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=heed`이면 snapshot과 문서 토큰이 `SNAPSHOT_HEED_PATH/store.json` repository-local shim의 `snapshots` named database에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 heed catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=hightower_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_HIGHTOWER_KV_PATH` hightower-kv 디렉터리의 `snapshot:<doc_id>` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 prefix scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=bitask`이면 snapshot과 문서 토큰이 `SNAPSHOT_BITASK_PATH` bitask append-only log 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay로 keydir를 재구축한 뒤 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=bitkv_rs`이면 snapshot과 문서 토큰이 `SNAPSHOT_BITKV_RS_PATH` bitkv-rs append-only log 디렉터리의 `doc_id -> persisted snapshot JSON` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay로 in-memory index를 재구축한 뒤 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=bitcask_engine`이면 snapshot과 문서 토큰이 `SNAPSHOT_BITCASK_ENGINE_PATH` bitcask-engine-rs append-only log 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` payload와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 log replay로 in-memory index를 재구축한 뒤 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=blazeup`이면 snapshot과 문서 토큰이 `SNAPSHOT_BLAZEUP_PATH` 디렉터리 아래 blazeup `snapshots` bucket의 `snapshot:<doc_id> -> persisted snapshot JSON string` record와 explicit `__catalog__` key에 저장된다. adapter는 blazeup의 process-global path 설정을 mutex로 직렬화하며, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=candystore`이면 snapshot과 문서 토큰이 `SNAPSHOT_CANDYSTORE_PATH` candystore 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. large payload는 `set_big/get_big` 경로를 사용하고, save/delete 뒤 `flush`와 `checkpoint`를 수행해 기본 local ownership 모드에서는 앱 시작 시 candystore catalog에서 room을 eager hydrate하며 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=celerix_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_CELERIX_STORE_PATH/snapshots.json` Celerix Store persona 파일의 `documents` app map에 `doc_id -> persisted snapshot JSON` value로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 app map catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=citadeldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CITADELDB_PATH` encrypted CitadelDB 파일의 `snapshots` table에 `doc_id -> persisted snapshot JSON bytes` entry와 explicit `__catalog__` key로 저장된다. 같은 경로의 `.citadel-keys` sidecar와 `SNAPSHOT_CITADELDB_PASSPHRASE`가 함께 필요하며, 기본 local ownership 모드에서는 catalog key에서 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=jammdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JAMMDB_PATH` jammdb 파일의 `snapshots` bucket에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 jammdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=mace`이면 snapshot과 문서 토큰이 `SNAPSHOT_MACE_PATH` Mace 디렉터리의 `snapshots` bucket에 `doc_id -> persisted snapshot JSON` 엔트리로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 Mace catalog key에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=janql`이면 snapshot과 문서 토큰이 `SNAPSHOT_JANQL_PATH` janql WAL/SSTable 디렉터리의 `doc_id -> persisted snapshot JSON` 엔트리와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 janql catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=jasondb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JASONDB_PATH` JasonDB append-only 파일의 `doc_id -> persisted snapshot JSON string` 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 JasonDB index replay catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=jasonisnthappy`이면 snapshot과 문서 토큰이 `SNAPSHOT_JASONISNTHAPPY_PATH` 단일 jasonisnthappy DB 파일의 `snapshots` collection에 `_id=<doc_id>` document로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=fjall`이면 snapshot과 문서 토큰이 `SNAPSHOT_FJALL_PATH` fjall DB 디렉터리의 `snapshots` keyspace에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 fjall catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=persy`이면 snapshot과 문서 토큰이 `SNAPSHOT_PERSY_PATH` persy 파일의 `snapshots` segment와 `snapshots_by_doc_id` index에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 persy catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=persistent_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_PERSISTENT_KV_PATH` persistent-kv 디렉터리의 snapshot set/WAL에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=native_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_NATIVE_DB_PATH` native_db 파일의 primary-key catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 native_db catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=nikidb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NIKIDB_PATH` nikidb 단일 파일의 `snapshots` bucket과 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 nikidb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=parity_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_PARITY_DB_PATH` parity-db shim 디렉터리의 repository-local `store.json` column map에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same persisted catalog iteration으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=redb`이면 snapshot과 문서 토큰이 `SNAPSHOT_REDB_PATH` redb 파일의 `snapshots` 테이블에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 redb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rskey`이면 snapshot과 문서 토큰이 `SNAPSHOT_RSKEY_PATH` rskey JSON hashmap 파일의 `doc_id -> persisted snapshot JSON string` 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rskey catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=readb`이면 snapshot과 문서 토큰이 `SNAPSHOT_READB_PATH` readb 디렉터리의 append-only data file과 index catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 readb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTLITE_PATH` rustlite 디렉터리의 WAL/SSTable engine과 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rustlite catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustcask`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTCASK_PATH` rustcask 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 저장된다. sync mode를 켜서 각 write 뒤 fsync를 보장하며, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하고 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=rusty_leveldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTY_LEVELDB_PATH` rusty-leveldb 디렉터리의 LevelDB keyspace에 `doc_id -> persisted snapshot JSON` key-value로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same keyspace full scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=canopydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CANOPYDB_PATH/store.json` repository-local canopydb shim의 `snapshots` named tree에 저장된다. commit은 temp-write + rename 뒤 optional file/dir sync로 확정되고, 기본 local ownership 모드에서는 앱 시작 시 같은 tree catalog를 eager hydrate한다. distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=caves`이면 snapshot과 문서 토큰이 `SNAPSHOT_CAVES_PATH` 디렉터리의 `<doc_id>` key-per-file 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 directory scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=ckydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CKYDB_PATH` ckydb 디렉터리의 explicit `__catalog__` key와 key-value 엔트리에 저장된다. payload와 catalog는 delimiter-safe write를 위해 base64 문자열로 저장되며, 기본 local ownership 모드에서는 앱 시작 시 ckydb catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=crepedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_CREPEDB_PATH` CrepeDB redb 파일의 basic `snapshots` table에 저장된다. payload는 `snapshot:<doc_id>` key에, 문서 목록은 explicit `__catalog__` key에 저장되며, 기본 local ownership 모드에서는 앱 시작 시 CrepeDB catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=crystal`이면 snapshot과 문서 토큰이 `SNAPSHOT_CRYSTAL_PATH` 디렉터리의 `<doc_id>.bin` bincode string file에 `persisted snapshot JSON`으로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 디렉터리 스캔으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=scdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SCDB_PATH` scdb 디렉터리의 explicit `__catalog__` key와 `doc_id` key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 scdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=skv`이면 snapshot과 문서 토큰이 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data`와 `<path>.index` 파일 쌍의 `doc_id` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=surrealkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_SURREALKV_PATH` surrealkv B+tree 단일 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 surrealkv full scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=pickledb`이면 snapshot과 문서 토큰이 `SNAPSHOT_PICKLEDB_PATH` PickleDB 파일의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 PickleDB catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rcask`이면 snapshot과 문서 토큰이 `SNAPSHOT_RCASK_PATH` RCask append-only log segment 디렉터리의 `doc_id` key와 explicit `__catalog__` key에 JSON string으로 저장된다. 공개 delete API가 없어 tombstone string으로 삭제를 가리며, 기본 local ownership 모드에서는 앱 시작 시 RCask catalog에서 room을 eager hydrate하고 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=microkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_MICROKV_PATH` base path에 대응하는 MicroKV 파일 `<path>.kv`의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 MicroKV catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=sled`이면 snapshot과 문서 토큰이 `SNAPSHOT_SLED_PATH` sled DB 디렉터리의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 sled catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustbreak`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTBREAK_PATH` rustbreak 단일 파일 catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rustbreak catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_YEDB_PATH` yedb-compatible 디렉터리의 `snapshots/<doc_id>` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=btree_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_BTREE_STORE_PATH` btree-store 단일 파일의 `snapshots` bucket key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 btree-store catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=cacache`이면 snapshot과 문서 토큰이 `SNAPSHOT_CACACHE_PATH` cacache content-addressed cache 디렉터리의 `snapshot:<doc_id>` key에 저장된다. 기본 local ownership 모드에서는 cache index listing으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
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
- `SNAPSHOT_STORE=saturn`이면 snapshot과 문서 토큰이 `SNAPSHOT_SATURN_PATH` SaturnDB WAL 파일의 `snapshot:<doc_id>` payload와 explicit `__catalog__` key에 JSON bytes로 저장된다. save/delete 뒤 WAL 파일과 parent directory를 sync하고, 기본 local ownership 모드에서는 앱 시작 시 WAL replay 뒤 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=snaildb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SNAILDB_PATH` snaildb 디렉터리의 key-value 엔트리와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 snaildb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=tinykv`이면 snapshot과 문서 토큰이 `SNAPSHOT_TINYKV_PATH` tinykv JSON 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 tinykv key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=vsdb`이면 `SNAPSHOT_VSDB_PATH/store.meta.json`에 store handle metadata가 저장되고, 실제 snapshot payload와 문서 토큰은 `VSDB_BASE_DIR` 또는 기본 `$HOME/.vsdb` 아래 `maps/<instance_id>.json` 파일에 `doc_id -> persisted snapshot JSON` catalog로 저장된다. 서버는 store 접근을 직렬화해 concurrent mutation을 막고, 기본 local ownership 모드에서는 same map full scan으로 room을 eager hydrate한다. distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yakv`이면 snapshot과 문서 토큰이 `SNAPSHOT_YAKV_PATH` yakv 단일 B-Tree 파일의 `snapshot:<doc_id>` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yakv full scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yakvdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_YAKVDB_PATH` yakvdb 단일 B-Tree 파일의 `snapshot:<doc_id>` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yakvdb key traversal catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=saberdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SABERDB_PATH` saberdb pretty JSON 파일의 `doc_id -> persisted snapshot JSON string` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=smolldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_SMOLLDB_PATH` compressed SmollDB 파일의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 파일을 load해 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=kstone`이면 snapshot과 문서 토큰이 `SNAPSHOT_KSTONE_PATH` repository-local kstone shim 디렉터리의 `wal.log` append-only log에 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key write/delete event로 저장된다. adapter는 startup 때 same log를 replay해 catalog와 payload keyspace를 복구하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate하며 distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=roughdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_ROUGHDB_PATH/store.json` repository-local roughdb shim map의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=raindb`이면 snapshot과 문서 토큰이 `SNAPSHOT_RAINDB_PATH` RainDB WAL/SSTable 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=infusedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_INFUSEDB_PATH` InfuseDB 단일 파일의 `snapshots` collection에 base64 text payload와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection을 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=kafi`이면 snapshot과 문서 토큰이 `SNAPSHOT_KAFI_PATH` 단일 bincode hashmap 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON string payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=feoxdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_FEOXDB_PATH` FeOxDB 단일 파일의 `snapshot:<doc_id>:<timestamp>:<event_id>` immutable event key와 tombstone event에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 prefix range scan으로 최신 event를 선택해 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=jsondb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JSONDB_PATH` jsondb versioned pretty JSON 파일의 `snapshots.<doc_id>` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=joydb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JOYDB_PATH` Joydb JSON state 파일의 `JoydbSnapshotRecord` catalog에 저장된다. save/delete 뒤 `flush()`를 호출하며, 기본 local ownership 모드에서는 앱 시작 시 Joydb JSON state를 load해 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=png_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_PNG_DB_PATH` 단일 PNG 파일의 compressed text row chunk에 저장된다. save/delete마다 전체 row set을 temp PNG로 교체하며, 기본 local ownership 모드에서는 앱 시작 시 PNG row scan으로 room을 eager hydrate하고 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=kopperdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_KOPPERDB_PATH` 디렉터리 아래 kopperdb append-only 세그먼트의 `doc_id` key와 explicit `__catalog__` key에 저장된다. delete API가 없어 tombstone value로 삭제를 가리고, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하며 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=icefalldb`이면 snapshot과 문서 토큰이 `SNAPSHOT_ICEFALLDB_PATH` 디렉터리의 `rsdb.log` append-only 로그에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 공개 delete API가 없어 tombstone value로 삭제를 가리고, 기본 local ownership 모드에서는 앱 시작 시 same catalog key를 읽어 room을 eager hydrate하며 distributed ownership 모드에서는 ownership 확인 이후 on-demand로 복구한다.
- `SNAPSHOT_STORE=eight`이면 snapshot과 문서 토큰이 `SNAPSHOT_EIGHT_PATH` 디렉터리 아래 eight filesystem storage의 `doc_<uuid_simple>` key tree에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 empty-prefix search catalog로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=epoch_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_EPOCH_DB_PATH` 디렉터리의 repository-local epoch-db shim `store.json` map에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=etchdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_ETCHDB_PATH` 디렉터리의 EtchDB WAL-backed store에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. save/delete는 `write_durable` 경계로 확정하고, 기본 local ownership 모드에서는 앱 시작 시 WAL replay 뒤 같은 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=fastkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_FASTKV_PATH` FastKV compressed binary dump 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. adapter는 save/delete마다 temp dump를 fsync한 뒤 rename하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=data_pile`이면 snapshot과 문서 토큰이 `SNAPSHOT_DATA_PILE_PATH` data-pile append-only record 디렉터리에 save/delete 이벤트로 저장된다. adapter는 startup 때 record log를 replay해 문서 catalog를 복구하고, save/delete 뒤 `data`/`seqno` 파일을 sync한다.
- `SNAPSHOT_STORE=ferrumdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_FERRUMDB_PATH` FerrumDB append-only log 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON value로 저장된다. save/delete 뒤 `FsyncPolicy::Always` 경계로 sync하고, 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=rumdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUMDB_PATH` 디렉터리의 append-only rumdb 로그 세트에 `doc_id` key와 explicit `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 같은 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=koit`이면 snapshot과 문서 토큰이 `SNAPSHOT_KOIT_PATH` koit structured JSON 파일의 `snapshots.<doc_id>` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=datastack`이면 snapshot과 문서 토큰이 `SNAPSHOT_DATASTACK_PATH` DataStack redb 파일의 `snapshots` collection에 `doc_id -> persisted snapshot JSON` document로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=lite_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_LITE_DB_PATH` LiteDb 디렉터리의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=log_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_LOG_KV_PATH` log_kv append-only 단일 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON string으로 저장된다. delete는 tombstone string으로 가리고, 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=append_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_APPEND_KV_PATH` append_kv append-only 단일 파일의 `snapshot:<doc_id>` key와 explicit `__catalog__` key에 JSON string으로 저장된다. delete는 append_kv tombstone record로 가리고, 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=append_log`이면 snapshot과 문서 토큰이 `SNAPSHOT_APPEND_LOG_PATH` append-log append-only 단일 파일에 save/delete JSON event로 저장된다. adapter는 startup 때 event log를 replay해 최신 snapshot map과 문서 catalog를 복구하고, save/delete 뒤 log 파일을 flush/sync한다.
- `SNAPSHOT_STORE=mhdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_MHDB_PATH` path prefix가 만드는 `<path>.pag`/`<path>.dir` DBM 파일 쌍에 chunked blob으로 저장된다. MHdb의 pair size 제한 때문에 adapter가 snapshot payload와 catalog를 작은 chunk key로 나눠 저장하고, 기본 local ownership 모드에서는 catalog blob을 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=loro_kv`이면 snapshot과 문서 토큰이 `SNAPSHOT_LORO_KV_PATH` 단일 export 파일의 `doc_id -> persisted snapshot JSON bytes` entry로 저장된다. save/delete마다 repository-local `loro-kv-store` shim의 whole-store export를 temp 파일에 쓰고 rename해 재시작 복구 경계를 고정한다.
- `SNAPSHOT_STORE=luckdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_LUCKDB_PATH` LuckDB JSON document 파일의 `backend.snapshots` collection에 `doc_id` field와 persisted snapshot JSON payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection query로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=ipjdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_IPJDB_PATH/snapshots/<item_id>` JSON item 파일에 `doc_id` field와 persisted snapshot JSON payload로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 collection full scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=kagi`이면 snapshot과 문서 토큰이 `SNAPSHOT_KAGI_PATH` 단일 kagi bincode hashmap 파일의 `doc_id -> persisted snapshot JSON string` entry로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file map load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=deeb`이면 snapshot과 문서 토큰이 `SNAPSHOT_DEEB_PATH` Deeb JSON database 파일의 `snapshots` entity에 `doc_id` primary key와 persisted snapshot JSON payload로 저장된다. save/delete마다 Deeb commit이 temp+rename으로 단일 파일을 갱신하고, 기본 local ownership 모드에서는 앱 시작 시 entity full scan으로 room을 eager hydrate한다.
- `SNAPSHOT_STORE=rubin`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUBIN_PATH` Rubin JSON 파일의 string map에 `doc_id -> persisted snapshot JSON` entry로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 JSON 파일을 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=lsm_engine`이면 snapshot과 문서 토큰이 `SNAPSHOT_LSM_ENGINE_PATH` lsm_engine WAL 파일에 `snapshot:<doc_id>` key와 explicit `__catalog__` key의 JSON string으로 저장된다. reopen 때 WAL을 replay해 in-memory LSM state를 재구성하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=lsm_storage_engine`이면 snapshot과 문서 토큰이 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. save/delete 뒤 `flush()`를 호출해 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=lsmdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_LSMDB_PATH` 디렉터리의 WAL/SSTable LSM keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. WAL sync-on-write 경계로 저장을 확정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=lsm_tree`이면 snapshot과 문서 토큰이 `SNAPSHOT_LSM_TREE_PATH` 디렉터리의 lsm-tree primitive keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. save/delete 뒤 active memtable을 flush해 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=mindb`이면 snapshot과 문서 토큰이 `SNAPSHOT_MINDB_PATH` 디렉터리의 repository-local mindb shim에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. current state는 `store.json`, replay log는 `wal.log`, manifest는 `manifest.json`에 유지되고 save/delete 뒤 `sync()`로 file-level durability 경계를 고정한다. 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=mmdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_MMDB_PATH/store.json` repository-local mmdb shim keyspace에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. sync write와 flush로 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=mu_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_MU_DB_PATH` data 파일과 같은 디렉터리의 `index_<file_name>` index 파일에 `snapshot:<doc_id>` key와 explicit `__catalog__` key로 저장된다. adapter는 save/delete 뒤 data/index 파일을 fsync하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=nanodb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NANODB_PATH` single JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry로 저장된다. save/delete 뒤 whole-file write로 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 whole-file object load로 room을 eager hydrate한다.
- `SNAPSHOT_STORE=jfs`이면 snapshot과 문서 토큰이 `SNAPSHOT_JFS_PATH` jfs single JSON 파일의 `doc_id -> persisted snapshot JSON string` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file catalog load로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=json_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_JSON_STORE_PATH` append-only JSON line 파일의 `doc_id -> persisted snapshot` catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file line replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=json_db_rs`이면 snapshot과 문서 토큰이 `SNAPSHOT_JSON_DB_RS_PATH` JSON 배열 이벤트 로그의 save/delete record에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 whole-file event replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=dir_cache`이면 snapshot과 문서 토큰이 `SNAPSHOT_DIR_CACHE_PATH` 디렉터리 아래 dir-cache entry의 `snapshot-<doc_id>.json` payload와 `__catalog__` key로 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog key를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=marble`이면 snapshot과 문서 토큰이 `SNAPSHOT_MARBLE_PATH` Marble object store의 persisted snapshot object와 catalog object에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 catalog object를 읽어 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=lmdb_rs_core`이면 snapshot과 문서 토큰이 `SNAPSHOT_LMDB_RS_CORE_PATH` lmdb-rs-core environment의 `snapshot:<doc_id>` payload와 explicit `__catalog__` key로 저장된다. save/delete commit 뒤 forced sync로 재시작 복구 경계를 고정하고, 기본 local ownership 모드에서는 catalog key를 읽어 room을 eager hydrate한다.
- `SNAPSHOT_STORE=hmdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_HMDB_PATH` 디렉터리 아래 hmdb schema 로그 파일의 `doc_id -> persisted snapshot` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 append-only 로그 replay로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=hurrahdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_HURRAHDB_PATH` 단일 AOF 파일의 `snapshot:<doc_id> -> persisted snapshot` payload와 explicit `__catalog__` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 AOF replay로 room을 eager hydrate하고, distributed ownership 모드에서는 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=fs_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_FS_DB_PATH/snapshot-<doc_id>.json` 파일에 저장된다. 기본 local ownership 모드에서는 디렉터리 file scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 ownership 확인 이후 on-demand로 같은 payload를 복원한다.
- `SNAPSHOT_STORE=sqjson`이면 snapshot과 문서 토큰이 `SNAPSHOT_SQJSON_PATH` single-file sqjson DB의 chunked `snapshot:<doc_id>` blob에 저장된다. 기본 local ownership 모드에서는 `snapshot:<doc_id>:meta` key scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 ownership 확인 이후 on-demand로 같은 blob을 복원한다.
- `SNAPSHOT_STORE=nodb`이면 snapshot과 문서 토큰이 `SNAPSHOT_NODB_PATH` nodb 단일 파일의 key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 nodb key scan catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=okofdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_OKOFDB_PATH` 디렉터리 아래 okofdb key-per-file storage의 `doc_<uuid_simple>` 파일 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same directory scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=simple_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_SIMPLE_DB_PATH` single-file simple_db store의 `doc_id -> base64(persisted snapshot JSON)` 라인 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=docdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_DOCDB_PATH` docdb JSON 파일의 `doc_id -> persisted snapshot` key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 same key scan으로 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=s3`이면 snapshot과 문서 토큰이 `SNAPSHOT_S3_ENDPOINT` / `SNAPSHOT_S3_BUCKET` / `SNAPSHOT_S3_PREFIX` 조합의 S3 object key `<prefix><doc_id>.json`에 저장된다. startup hydrate는 bucket listing 뒤 각 object를 읽어 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=managed`이면 snapshot과 문서 토큰이 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external snapshot service `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`를 통해 저장된다. 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SqliteSnapshotStore`는 row-level upsert로 기존 snapshot을 교체하며, 잘못된 timestamp나 손상된 row는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `HeedSnapshotStore`는 repository-local `store.json` shim의 named database upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `JammdbSnapshotStore`는 single-file B+ tree bucket upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `JanqlSnapshotStore`는 WAL/SSTable 디렉터리 keyspace upsert와 explicit `__catalog__` key를 함께 사용해 기존 snapshot을 교체하며, 손상된 snapshot payload나 missing catalog entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `CandystoreSnapshotStore`는 directory-backed append-only engine에 large payload를 `set_big/get_big`로 저장하고 `__catalog__` key를 별도로 유지하며, `flush`와 `checkpoint` 뒤 기존 snapshot을 교체한다. 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `FjallSnapshotStore`는 LSM-tree keyspace upsert 뒤 `PersistMode::SyncAll`로 journal을 동기화해 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `PersySnapshotStore`는 single-file copy-on-write segment update와 `doc_id -> record_id` replace index를 함께 사용해 기존 snapshot을 교체하며, 손상된 snapshot payload나 dangling index entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `ParityDbSnapshotStore`는 repository-local parity-db shim의 persisted column map upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `RedbSnapshotStore`는 key-value upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `MindbSnapshotStore`는 WAL/SSTable LSM keyspace upsert와 explicit `__catalog__` key를 함께 사용해 기존 snapshot을 교체하고 save/delete 뒤 `sync()`로 WAL durability 경계를 고정한다. reopen point index가 비어 있는 경우 upstream `RecoveryManager` WAL replay fallback으로 catalog/snapshot을 읽으며, 손상된 snapshot payload나 missing catalog entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- 기본 `LocalRoomLocator`는 모든 문서를 현재 프로세스 소유로 해석한다.
- `StaticRoomLocator`는 `ROOM_OWNER_HINTS_PATH`의 문서별 owner 힌트를 읽고, 현재 `NODE_ID`와 다른 owner를 가진 문서에 대해 `409 conflict`와 owner 힌트를 반환한다.
- `FileRoomLocator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `SqliteRoomLocator`는 `ROOM_COORDINATOR_SQLITE_PATH`의 repository-local sqlite shim 파일 안 logical `room_leases` row를 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `ROOM_COORDINATOR=noop`은 아무 side effect 없이 통과하고, `ROOM_COORDINATOR=logging`은 `NODE_ID`와 `doc_id` 기준 lifecycle log만 남긴다.
- `ROOM_COORDINATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`에 canonical lease state (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 atomic write로 남기고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release 방식으로 정리한다. `NODE_BASE_URL`이 주어지면 이 값도 canonical origin으로 정규화해 함께 기록한다.
- `ROOM_COORDINATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 repository-local sqlite shim 파일 안 logical `room_leases` table에 같은 canonical lease state를 upsert하고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 `node_id + lease_id + epoch` compare-and-delete로 정리한다. write/read 경계는 같은 `<path>.lock` sidecar lock으로 serialize된다. `NODE_BASE_URL`이 주어지면 canonical origin으로 정규화한 `base_url`도 함께 기록한다.
- `ROOM_LOCATOR=file`과 `ROOM_COORDINATOR=file`은 같은 `ROOM_COORDINATOR_STATE_DIR`를 공유해야 하며, 멀티 노드에서 쓰려면 각 노드가 같은 디렉터리를 읽고 쓸 수 있어야 한다.
- `ROOM_LOCATOR=sqlite`와 `ROOM_COORDINATOR=sqlite`는 같은 `ROOM_COORDINATOR_SQLITE_PATH`와 `<path>.lock` sidecar를 공유해야 하며, 실제 owner handoff를 원하면 shared snapshot store도 함께 맞춰야 한다.
- WebSocket 첫 세션 시작과 마지막 세션 종료 시점에 `RoomCoordinator` hook이 호출되도록 런타임 경계가 이미 연결돼 있다.
- 현재 file-backed lease state는 shared filesystem 위에서만 동작하는 best-effort 구현이다. crash 뒤에는 `expires_at` 경과 후에만 stale로 간주된다.
- `SqliteRoomCoordinator`/`SqliteRoomLocator`는 shared sqlite shim 파일에서 sidecar lock 기반 lease compare-and-swap을 수행한다. 실제 owner handoff는 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store와 함께 구성했을 때만 안전하게 활성화해야 한다.
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
- 현재 저장소의 `SqliteRoomCoordinator`/`SqliteRoomLocator`는 같은 계약을 shared sqlite shim file row에 매핑한 authoritative CAS 구현을 제공한다.
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
