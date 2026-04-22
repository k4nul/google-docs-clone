# API

## Error Response Shape

입력 검증 실패, 인증 실패, 접근 거절은 다음 JSON 구조로 반환된다.

```json
{
  "error": "bad_request",
  "message": "id must be a valid UUID, received `not-a-uuid`"
}
```

room ownership conflict처럼 non-local owner 힌트를 함께 주는 경우에는 `owner` 객체가 추가될 수 있다.

```json
{
  "error": "conflict",
  "message": "document `00000000-0000-0000-0000-000000000000` is owned by another collaboration node",
  "owner": {
    "node_id": "node-b",
    "base_url": "http://127.0.0.1:5001"
  }
}
```

- authoritative coordination resolver도 같은 `owner.node_id` / optional `owner.base_url` shape를 유지해야 한다. 현재 저장소의 `ROOM_LOCATOR=sqlite|managed`도 이 shape를 그대로 사용한다.
- `owner.base_url`이 존재하면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하고, 응답에는 canonical origin (`scheme://authority`)으로 반환한다.
- 같은 non-local owner conflict 응답은 JSON body 외에도 `x-collab-owner-node-id` 헤더를 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함한다.
- `x-collab-redirect-location`과 `Location` 값은 owner origin 뒤에 현재 요청 path/query를 그대로 붙인 absolute URL이어야 한다. 예를 들어 `GET /ws/:doc_id?source=edge`가 remote owner로 거절되면 `Location: https://node-b.internal/ws/:doc_id?source=edge` 형태가 된다.

## HTTP Endpoints

### `GET /api/health`

Response:

```json
{
  "status": "ok",
  "service": "backend",
  "timestamp": "2026-04-17T14:00:00Z"
}
```

### `GET /api/documents`

- `Authorization: Bearer <API_TOKEN>` 헤더가 필요하다.

Response:

```json
{
  "documents": [
    {
      "id": "00000000-0000-0000-0000-000000000000",
      "title": "Document 00000000-0000-0000-0000-000000000000",
      "created_at": "2026-04-17T14:00:00Z",
      "updated_at": "2026-04-17T14:00:00Z"
    }
  ]
}
```

active room과 snapshot store에 남아 있는 persisted document catalog를 합쳐 문서 목록을 반환한다.
- snapshot store는 현재 `file`, `agdb`, `amandine`, `armdb`, `flash_kv`, `blockbucket`, `grebedb`, `grumpydb`, `graus_db`, `highlandcows_isam`, `simple_db`, `docdb`, `eight`, `epoch_db`, `ferrumdb`, `rumdb`, `rubin`, `shorterdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `cuendillar`, `jammdb`, `mace`, `janql`, `jasondb`, `jasonisnthappy`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `rcask`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `sanakirja`, `snaildb`, `tinykv`, `vsdb`, `yakv`, `saberdb`, `smolldb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `jsondb`, `kopperdb`, `kv`, `koit`, `lite_db`, `log_kv`, `mhdb`, `loro_kv`, `luckdb`, `lsm_engine`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `nanodb`, `jfs`, `json_store`, `feoxdb`, `s3`, `managed` durability backend를 지원한다. `mace` 모드에서는 `SNAPSHOT_MACE_PATH` 디렉터리의 Mace `snapshots` bucket과 explicit `__catalog__` key에서 catalog를 복구하고, `janql` 모드에서는 `SNAPSHOT_JANQL_PATH` 디렉터리의 janql WAL/SSTable keyspace에서 `doc_id` key와 explicit `__catalog__` key를 읽어 catalog를 복구한다.
- `cuendillar` 모드에서는 `SNAPSHOT_CUENDILLAR_PATH` 루트 아래 `wal/`과 `sstable/` 디렉터리를 사용하는 LSM engine keyspace 전체 scan으로 catalog를 복구하고, restart recovery를 위해 WAL payload 상한과 sync policy를 보수적으로 높여 둔다.
- `celerix_store` 모드에서는 `SNAPSHOT_CELERIX_STORE_PATH/snapshots.json` Celerix Store persona 파일의 `documents` app map에 `doc_id -> persisted snapshot JSON` value를 저장하고, catalog는 같은 app map key를 순회해 복구한다.
- `kopperdb` 모드에서는 `SNAPSHOT_KOPPERDB_PATH` 디렉터리 아래 append-only segment log를 사용해 `doc_id` key와 explicit `__catalog__` key를 유지한다. delete API가 없어 tombstone string을 덮어써 삭제를 가린다.
- `rcask` 모드에서는 `SNAPSHOT_RCASK_PATH` 디렉터리 아래 append-only segment log를 사용해 `doc_id` key와 explicit `__catalog__` key를 JSON string으로 유지한다. 공개 delete API가 없어 tombstone string을 덮어써 삭제를 가린다.
- `icefalldb` 모드에서는 `SNAPSHOT_ICEFALLDB_PATH` 디렉터리 아래 append-only `rsdb.log`를 사용해 `doc_id` key와 explicit `__catalog__` key를 유지한다. 공개 delete API가 없어 tombstone value를 덮어써 삭제를 가린다.

### `POST /api/documents`

- `Authorization: Bearer <API_TOKEN>` 헤더가 필요하다.

Request body:

```json
{
  "title": "Design notes"
}
```

- `title`은 선택값이다.
- `title`이 비어 있거나 누락되면 기본 제목 `Document {uuid}`를 사용한다.
- 서버가 새 UUID를 생성하고 해당 문서 room을 메모리 및 snapshot store에 등록한다.
- 응답의 `credentials.access_token`은 이후 문서 상세 조회, 삭제, WebSocket 연결에 사용한다.

Response: `201 Created`

```json
{
  "document": {
    "id": "00000000-0000-0000-0000-000000000000",
    "title": "Design notes",
    "created_at": "2026-04-17T14:00:00Z",
    "updated_at": "2026-04-17T14:00:00Z"
  },
  "credentials": {
    "access_token": "11111111-1111-1111-1111-111111111111"
  }
}
```

### `GET /api/documents/:id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- Path parameter `id`는 UUID 형식이어야 한다.
- 현재 노드 ownership을 `RoomLocator` 경계로 먼저 확인하고, active room이 없으면 snapshot store에서 문서를 on-demand로 복구한다.
- snapshot restore source는 현재 `SNAPSHOT_STORE=file|armdb|flash_kv|blockbucket|grebedb|grumpydb|graus_db|highlandcows_isam|simple_db|docdb|eight|epoch_db|rumdb|shorterdb|sqlite|heed|hightower_kv|hmdb|icefalldb|bitask|bitkv_rs|bitcask_engine|blazeup|candystore|celerix_store|cuendillar|jammdb|mace|fjall|persy|persistent_kv|native_db|nebari|nikidb|nodb|okofdb|parity_db|pickledb|rcask|microkv|redb|rskey|readb|rustlite|rustcask|rusty_leveldb|canopydb|caves|ckydb|crepedb|scdb|skv|surrealkv|sled|rustbreak|yedb|btree_store|siamesedb|structsy|abyssiniandb|aeternusdb|thunderdb|thetadb|tinybase|tinydb|dblite|dbless|db_rs|dharmadb|sanakirja|snaildb|tinykv|yakv|saberdb|smolldb|kstone|jsondb|joydb|kopperdb|kv|koit|lite_db|log_kv|mhdb|loro_kv|luckdb|rubin|lsm_engine|lsm_storage_engine|lsmdb|lsm_tree|mindb|mmdb|nanodb|jfs|json_store|feoxdb|s3|managed` 중 하나다.
- 문서가 없으면 `404` JSON 에러를 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403`을 반환한다.
- `ROOM_LOCATOR=static`, `ROOM_LOCATOR=file`, `ROOM_LOCATOR=sqlite`, `ROOM_LOCATOR=managed`, 또는 동등한 authoritative resolver가 현재 노드 비소유를 보고하면 local restore 대신 `409` JSON 에러로 중단한다. 이때 owner 힌트가 있으면 `owner.node_id`와 optional `owner.base_url`를 함께 반환한다. 기본 `LocalRoomLocator` 구성에서는 이 경로가 발생하지 않는다.
- `ROOM_OWNER_HINTS_PATH`에 선언하는 `owner.node_id`와 `owner.base_url`은 trim 후 저장된다.
- `owner.base_url`은 선택값이지만, 사용할 경우 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하며 응답에는 canonical origin (`scheme://authority`) 형태로 반환된다.
- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active owner lease state를 읽는다. 해당 state에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 읽는다. 해당 row에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL`의 `GET /v1/leases/:doc_id`를 읽는다. 해당 lease record에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- non-local owner conflict 응답은 `x-collab-owner-node-id` 헤더를 항상 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함하고, redirect URL은 현재 요청의 path/query를 그대로 유지한다.
- `ROOM_LOCATOR=file`과 `ROOM_LOCATOR=sqlite`는 persisted `expires_at`이 지나기 전까지 다른 node lease를 authoritative하게 취급하고, 만료 뒤에만 stale owner로 간주한다.
- `ROOM_LOCATOR=managed`를 포함한 authoritative coordination resolver는 stale 판단을 `expires_at` 기반 lease 만료로 수행해야 하며, 그 결과를 동일한 `409` owner metadata shape로 노출해야 한다.
- UUID 형식이 아니면 `400`과 JSON 에러 응답을 반환한다.

Response:

```json
{
  "document": {
    "id": "00000000-0000-0000-0000-000000000000",
    "title": "Design notes",
    "created_at": "2026-04-17T14:00:00Z",
    "updated_at": "2026-04-17T14:00:00Z"
  }
}
```

### `DELETE /api/documents/:id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- Path parameter `id`는 UUID 형식이어야 한다.
- 문서가 존재하면 room과 문서 메타데이터를 함께 제거한다.
- 문서가 없으면 `404` JSON 에러 응답을 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403`을 반환한다.

If an active collaboration WebSocket session is still attached to the document, the delete request returns `409 Conflict` with the standard JSON error shape.

Response: `204 No Content`

## WebSocket Path

### `GET /ws/:doc_id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- `doc_id`는 UUID 형식이어야 한다.
- 문서는 먼저 `POST /api/documents`로 생성되어 있어야 한다.
- WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 한다.
- 같은 `doc_id`를 사용하는 클라이언트는 같은 Yrs broadcast group에 연결된다.
- 현재 노드 ownership을 `RoomLocator` 경계로 먼저 확인하고, active room이 없으면 snapshot store에서 room을 on-demand로 복구한다.
- snapshot restore source는 현재 `SNAPSHOT_STORE=file|armdb|flash_kv|blockbucket|grebedb|grumpydb|graus_db|highlandcows_isam|simple_db|docdb|eight|epoch_db|rumdb|shorterdb|sqlite|heed|hightower_kv|hmdb|icefalldb|bitask|bitkv_rs|bitcask_engine|blazeup|candystore|celerix_store|cuendillar|jammdb|mace|fjall|persy|persistent_kv|native_db|nebari|nikidb|nodb|okofdb|parity_db|pickledb|rcask|microkv|redb|rskey|readb|rustlite|rustcask|rusty_leveldb|canopydb|caves|ckydb|crepedb|scdb|skv|surrealkv|sled|rustbreak|yedb|btree_store|siamesedb|structsy|abyssiniandb|aeternusdb|thunderdb|thetadb|tinybase|tinydb|dblite|dbless|db_rs|dharmadb|sanakirja|snaildb|tinykv|yakv|saberdb|smolldb|kstone|jsondb|joydb|kopperdb|kv|koit|lite_db|log_kv|mhdb|loro_kv|luckdb|rubin|lsm_engine|lsm_storage_engine|lsmdb|lsm_tree|mindb|mmdb|nanodb|jfs|json_store|feoxdb|s3|managed` 중 하나다.
- 내부 `RoomCoordinator` hook은 `ROOM_COORDINATOR` 설정에 따라 `noop`, `logging`, `file`, `sqlite`, 또는 `managed` 모드로 동작하며, 현재 단계에서는 HTTP/WS 계약 자체를 바꾸지 않는다.
- 마지막 WebSocket 세션이 종료되면 최신 snapshot을 저장한 뒤 idle room을 메모리에서 제거한다.
- `doc_id`가 UUID 형식이 아니면 `400` JSON 에러 응답을 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403` JSON 에러 응답을 반환한다.
- 문서가 존재하지 않으면 업그레이드 전에 `404` JSON 에러 응답을 반환한다.
- `Origin` 헤더가 없거나 허용되지 않으면 업그레이드 전에 `403` JSON 에러 응답을 반환한다.
- `ROOM_LOCATOR=static`, `ROOM_LOCATOR=file`, `ROOM_LOCATOR=sqlite`, `ROOM_LOCATOR=managed`, 또는 동등한 authoritative resolver가 현재 노드 비소유를 보고하면 업그레이드 전에 `409` JSON 에러 응답을 반환한다. 이때 owner 힌트가 있으면 `owner.node_id`와 optional `owner.base_url`를 함께 반환한다. 기본 `LocalRoomLocator` 구성에서는 이 경로가 발생하지 않는다.
- `ROOM_OWNER_HINTS_PATH`에 선언하는 `owner.node_id`와 `owner.base_url`은 trim 후 저장된다.
- `owner.base_url`은 선택값이지만, 사용할 경우 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하며 응답에는 canonical origin (`scheme://authority`) 형태로 반환된다.
- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active owner lease state를 읽는다. 해당 state에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 active owner lease row를 읽는다. 해당 row에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL`의 `GET /v1/leases/:doc_id` 응답을 읽는다. 해당 lease record에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- non-local owner conflict 응답은 `x-collab-owner-node-id` 헤더를 항상 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함하고, redirect URL은 현재 요청의 path/query를 그대로 유지한다.
- `ROOM_COORDINATOR=file`은 첫 active session에서 file-backed lease를 acquire하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 compare-and-release로 lease를 정리한다.
- `ROOM_COORDINATOR=sqlite`는 첫 active session에서 SQLite-backed lease row를 acquire하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 `node_id + lease_id + epoch` compare-and-delete로 lease를 정리한다.
- `ROOM_COORDINATOR=managed`는 첫 active session에서 managed lease service `POST /v1/leases/:doc_id/acquire`를 호출하고, background heartbeat로 `POST /v1/leases/:doc_id/renew`를 반복하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 `POST /v1/leases/:doc_id/release`로 compare-and-release를 요청한다.
- `ROOM_LOCATOR=file|sqlite|managed`와 동등한 authoritative coordination resolver는 모두 lease 만료 전까지 기존 owner를 authoritative하게 취급하고, `expires_at` 경과 뒤에만 ownership handoff를 허용해야 한다.

## Frontend Contract Notes

## Snapshot Durability Notes

- `SNAPSHOT_STORE=s3`는 S3-compatible object storage durability backend다.
- `SNAPSHOT_STORE=agdb`는 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=armdb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=blockbucket`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=grebedb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=grumpydb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=graus_db`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=heed`는 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=jammdb`는 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=fjall`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=persy`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=native_db`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=parity_db`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=redb`는 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rskey`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=readb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rustlite`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rustcask`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rumdb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rusty_leveldb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=canopydb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=crepedb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=scdb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=skv`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=surrealkv`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=pickledb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=microkv`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=sled`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=rustbreak`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=yedb`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=btree_store`도 vendor-specific embedded database durability backend다.
- `SNAPSHOT_STORE=agdb`, `SNAPSHOT_STORE=armdb`, `SNAPSHOT_STORE=flash_kv`, `SNAPSHOT_STORE=blockbucket`, `SNAPSHOT_STORE=grebedb`, `SNAPSHOT_STORE=grumpydb`, `SNAPSHOT_STORE=graus_db`, `SNAPSHOT_STORE=simple_db`, `SNAPSHOT_STORE=docdb`, `SNAPSHOT_STORE=eight`, `SNAPSHOT_STORE=epoch_db`, `SNAPSHOT_STORE=ferrumdb`, `SNAPSHOT_STORE=rumdb`, `SNAPSHOT_STORE=shorterdb`, `SNAPSHOT_STORE=siamesedb`, `SNAPSHOT_STORE=structsy`, `SNAPSHOT_STORE=abyssiniandb`, `SNAPSHOT_STORE=aeternusdb`, `SNAPSHOT_STORE=thunderdb`, `SNAPSHOT_STORE=thetadb`, `SNAPSHOT_STORE=tinybase`, `SNAPSHOT_STORE=tinydb`, `SNAPSHOT_STORE=dblite`, `SNAPSHOT_STORE=dbless`, `SNAPSHOT_STORE=db_rs`, `SNAPSHOT_STORE=dharmadb`, `SNAPSHOT_STORE=sanakirja`, `SNAPSHOT_STORE=snaildb`, `SNAPSHOT_STORE=tinykv`, `SNAPSHOT_STORE=saberdb`, `SNAPSHOT_STORE=smolldb`, `SNAPSHOT_STORE=kstone`, `SNAPSHOT_STORE=roughdb`, `SNAPSHOT_STORE=raindb`, `SNAPSHOT_STORE=infusedb`, `SNAPSHOT_STORE=kafi`, `SNAPSHOT_STORE=tinkv`, `SNAPSHOT_STORE=ledger_kv`, `SNAPSHOT_STORE=jsondb`, `SNAPSHOT_STORE=joydb`, `SNAPSHOT_STORE=kopperdb`, `SNAPSHOT_STORE=icefalldb`, `SNAPSHOT_STORE=bitkv_rs`, `SNAPSHOT_STORE=bitcask_engine`, `SNAPSHOT_STORE=blazeup`, `SNAPSHOT_STORE=kv`, `SNAPSHOT_STORE=koit`, `SNAPSHOT_STORE=lite_db`, `SNAPSHOT_STORE=log_kv`, `SNAPSHOT_STORE=mhdb`, `SNAPSHOT_STORE=loro_kv`, `SNAPSHOT_STORE=luckdb`, `SNAPSHOT_STORE=lsm_engine`, `SNAPSHOT_STORE=lsm_storage_engine`, `SNAPSHOT_STORE=lsmdb`, `SNAPSHOT_STORE=lsm_tree`, `SNAPSHOT_STORE=mindb`, `SNAPSHOT_STORE=mmdb`, `SNAPSHOT_STORE=nanodb`, `SNAPSHOT_STORE=jfs`, `SNAPSHOT_STORE=json_store`, `SNAPSHOT_STORE=persistent_kv`, `SNAPSHOT_STORE=nebari`, `SNAPSHOT_STORE=nikidb`, `SNAPSHOT_STORE=nodb`, `SNAPSHOT_STORE=okofdb`, `SNAPSHOT_STORE=caves`, `SNAPSHOT_STORE=ckydb`, `SNAPSHOT_STORE=crepedb`, `SNAPSHOT_STORE=hightower_kv`, `SNAPSHOT_STORE=rustcask`, `SNAPSHOT_STORE=skv`도 vendor-specific embedded database durability backend다.
- 필수 env는 `SNAPSHOT_AGDB_PATH`다.
- agdb catalog는 단일 graph DB 파일의 `snapshot:<doc_id>` alias node payload key에 persisted snapshot JSON string을 저장하고, `GET /api/documents` catalog는 all-alias scan 뒤 matching alias node를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_ARMDB_PATH`다.
- armdb catalog는 디렉터리 기반 sharded VarTree에 UUID bytes key와 persisted snapshot JSON bytes value를 저장하고, `GET /api/documents` catalog는 tree iteration 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BLOCKBUCKET_PATH`다.
- blockbucket bucket 파일은 `doc_id -> persisted snapshot JSON` raw bytes entry를 저장하고, `GET /api/documents` catalog는 `list_next` pagination으로 keyspace를 끝까지 스캔해 각 payload를 복원한다.
- 필수 env는 `SNAPSHOT_TINYBASE_PATH`다.
- tinybase `snapshots` typed table은 `doc_id` secondary index와 constant catalog index를 함께 유지하고, `GET /api/documents` catalog는 해당 secondary index query 뒤 최신 record만 선택해 문서 메타데이터를 복원한다.
- 필수 env는 `SNAPSHOT_TINYDB_PATH`다.
- tinydb catalog는 bincode 단일 파일 안의 `doc_id` keyed record를 저장하고, `GET /api/documents` catalog는 tinydb dump load 뒤 record payload를 다시 읽어 문서 메타데이터를 복원한다.
- 필수 env는 `SNAPSHOT_HEED_PATH`다.
- heed LMDB `snapshots` database는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JAMMDB_PATH`다.
- jammdb `snapshots` bucket은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_MACE_PATH`다.
- Mace `snapshots` bucket은 `doc_id -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JANQL_PATH`다.
- janql WAL/SSTable keyspace는 `doc_id -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JASONDB_PATH`다.
- JasonDB append-only 파일은 `doc_id -> persisted snapshot JSON string` entry를 저장하고, `GET /api/documents` catalog는 startup index replay 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JASONISNTHAPPY_PATH`다.
- jasonisnthappy `snapshots` collection은 `_id=<doc_id>` document에 persisted snapshot JSON payload를 저장하고, `GET /api/documents` catalog는 collection scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_FJALL_PATH`다.
- fjall `snapshots` keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_PERSY_PATH`다.
- persy `snapshots` segment와 `snapshots_by_doc_id` replace index는 `doc_id -> persisted snapshot JSON record` 매핑을 저장하고, `GET /api/documents` catalog는 전체 index scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_NATIVE_DB_PATH`다.
- native_db primary-key catalog는 `doc_id -> persisted snapshot JSON` payload를 저장하고, `GET /api/documents` catalog는 전체 scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_NEBARI_PATH`다.
- nebari `snapshots` tree는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 tree range scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_PARITY_DB_PATH`다.
- parity-db ordered `snapshots` column은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 ordered scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_REDB_PATH`다.
- redb `snapshots` 테이블은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RSKEY_PATH`다.
- rskey JSON hashmap은 `doc_id -> persisted snapshot JSON string` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_READB_PATH`다.
- readb keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RUSTLITE_PATH`다.
- rustlite keyspace는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RUSTY_LEVELDB_PATH`다.
- rusty-leveldb keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 same keyspace full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CANOPYDB_PATH`다.
- canopydb `snapshots` tree는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 tree iter scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CKYDB_PATH`다.
- ckydb key-value store는 `doc_id -> base64(persisted snapshot JSON)` 문자열 엔트리를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CREPEDB_PATH`다.
- CrepeDB basic table은 `snapshot:<doc_id> -> persisted snapshot JSON` binary 엔트리를 저장하고, `GET /api/documents` catalog는 같은 table의 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SCDB_PATH`다.
- scdb key-value store는 `doc_id -> persisted snapshot JSON` binary 엔트리를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SKV_PATH`다.
- skv key-value store는 `SNAPSHOT_SKV_PATH` base path가 만드는 `<path>.data` / `<path>.index` 파일 쌍에 `doc_id -> persisted snapshot JSON` payload와 별도 `__catalog__` key를 저장하고, `GET /api/documents` catalog는 해당 보조 key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SURREALKV_PATH`다.
- surrealkv B+tree keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_PICKLEDB_PATH`다.
- PickleDB는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RCASK_PATH`다.
- RCask는 append-only segment log에 `doc_id -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 저장하고, 공개 delete API가 없어 tombstone string으로 삭제를 가린다. `GET /api/documents` catalog는 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_MICROKV_PATH`다.
- MicroKV는 파일 `<path>.kv`에 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SLED_PATH`다.
- sled DB는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RUSTBREAK_PATH`다.
- rustbreak path database catalog는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_YEDB_PATH`다.
- yedb catalog는 `snapshots/<doc_id>` key에 `persisted snapshot JSON`을 저장하고, `GET /api/documents` catalog는 `snapshots` namespace 전체 scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BTREE_STORE_PATH`다.
- btree-store `snapshots` bucket은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 전체 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SIAMESDB_PATH`다.
- siamesedb `snapshots` map은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 보조 catalog key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_STRUCTSY_PATH`다.
- structsy persistent record는 `doc_id`, title/timestamps/token, Yrs full-state update 필드를 저장하고, `GET /api/documents` catalog는 record scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_ABYSSINIANDB_PATH`다.
- abyssiniandb `snapshots` map은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_AETERNUSDB_PATH`다.
- aeternusdb keyspace는 `doc_id -> persisted snapshot JSON` binary value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_THUNDERDB_PATH`다.
- thunderdb `snapshots` bucket은 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 bucket iter scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_THETADB_PATH`다.
- thetadb keyspace는 raw `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 cursor full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_GREBEDB_PATH`다.
- grebedb keyspace는 `doc_id -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 함께 저장하고, `GET /api/documents` catalog는 보조 catalog key를 따라 각 payload를 복원한다. save/delete는 같은 `flush()` 경계로 반영된다.
- 필수 env는 `SNAPSHOT_GRUMPYDB_PATH`다.
- grumpydb page/B+Tree object store는 UUID key와 persisted snapshot JSON bytes value를 저장하고, `GET /api/documents` catalog는 full range scan 뒤 각 payload를 복원한다.
- 필수 env는 `SNAPSHOT_GRAUS_DB_PATH`다.
- GrausDb keyspace는 `doc_id -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다. save/delete 뒤 DB handle을 재오픈해 buffered writer flush와 log replay 복구 경계를 고정한다.
- 필수 env는 `SNAPSHOT_VSDB_PATH`다.
- vsdb `Mapx<String, Vec<u8>>`는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 process-global keyspace full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다. 재기동 복구에는 `SNAPSHOT_VSDB_PATH/store.meta.json` handle metadata가 함께 필요하다.
- 필수 env는 `SNAPSHOT_DBLITE_PATH`다.
- dblite string key-value store는 `doc_id -> persisted snapshot JSON` bytes 엔트리를 저장하고, `GET /api/documents` catalog는 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_DBLESS_PATH`다.
- dbless typed table store는 `doc_id -> persisted snapshot` 엔트리를 저장하고, `GET /api/documents` catalog는 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_DB_RS_PATH`다.
- db-rs typed table store는 `LookupTable<String, PersistedSnapshot>` append-only 로그 엔트리에 `doc_id -> persisted snapshot`을 저장하고, `GET /api/documents` catalog는 로그 replay 뒤 같은 table key scan으로 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SANAKIRJA_PATH`다.
- sanakirja keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_FLASH_KV_PATH`다.
- flash-kv keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다. save/delete 뒤 `sync()`를 호출해 재시작 복구 경계를 고정한다.
- 필수 env는 `SNAPSHOT_SIMPLE_DB_PATH`다.
- simple_db single-file store는 `doc_id -> base64(persisted snapshot JSON)` 라인 엔트리를 저장하고, `GET /api/documents` catalog는 same key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_DOCDB_PATH`다.
- docdb single-file store는 `doc_id -> persisted snapshot` key-value 엔트리를 저장하고, `GET /api/documents` catalog는 same key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SHORTERDB_PATH`다.
- shorterdb keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SNAILDB_PATH`다.
- snaildb keyspace는 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 별도 `__catalog__` key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_TINYKV_PATH`다.
- tinykv keyspace는 `doc_id -> persisted snapshot JSON string` key-value를 저장하고, `GET /api/documents` catalog는 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_YAKV_PATH`다.
- yakv keyspace는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SABERDB_PATH`다.
- saberdb catalog는 `doc_id -> persisted snapshot JSON string` key-value를 pretty JSON 파일 하나에 저장하고, `GET /api/documents` catalog는 whole-file map load 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_SMOLLDB_PATH`다.
- smolldb catalog는 compressed single-file SmollDB map에 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 file load 뒤 catalog key를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_KSTONE_PATH`다.
- kstone catalog는 Kstone WAL/SSTable LSM 디렉터리 keyspace의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_ROUGHDB_PATH`다.
- roughdb catalog는 LevelDB-compatible WAL/SSTable 디렉터리 keyspace의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key를 sync write batch로 함께 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RAINDB_PATH`다.
- raindb catalog는 LevelDB-style WAL/SSTable 디렉터리 keyspace의 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key를 synchronous write batch로 함께 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_INFUSEDB_PATH`다.
- infusedb catalog는 InfuseDB 단일 파일의 `snapshots` collection에 base64 text로 인코딩된 `snapshot:<doc_id> -> persisted snapshot JSON bytes`와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_KAFI_PATH`다.
- kafi catalog는 단일 bincode hashmap 파일의 `snapshot:<doc_id> -> persisted snapshot JSON string` payload와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_TINKV_PATH`다.
- tinkv catalog는 append-only data file 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_LEDGER_KV_PATH`다.
- ledger_kv catalog는 append-only ledger 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_FEOXDB_PATH`다.
- feoxdb catalog는 단일 FeOxDB 파일에 `snapshot:<doc_id>:<timestamp>:<event_id> -> persisted snapshot JSON` immutable event와 tombstone event를 저장하고, `GET /api/documents` catalog는 prefix range scan 뒤 최신 event를 선택해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_DHARMADB_PATH`다.
- dharmadb catalog는 WAL/SSTable 디렉터리 keyspace의 `doc_id -> persisted snapshot JSON string` 엔트리와 explicit `__catalog__` key를 함께 저장하고, `GET /api/documents` catalog는 worker thread를 통해 catalog key를 읽은 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JSONDB_PATH`다.
- jsondb catalog는 versioned pretty JSON 파일의 `snapshots.<doc_id>` key-value를 저장하고, `GET /api/documents` catalog는 whole-file map load 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JOYDB_PATH`다.
- joydb catalog는 Joydb JSON state 파일의 `JoydbSnapshotRecord` table에 `doc_id -> persisted snapshot` record를 저장하고, `GET /api/documents` catalog는 state load 뒤 각 record payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_EIGHT_PATH`다.
- eight catalog는 filesystem storage의 `doc_<uuid_simple> -> persisted snapshot JSON string` key-value를 저장하고, `GET /api/documents` catalog는 empty-prefix search 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_EPOCH_DB_PATH`다.
- epoch-db catalog는 sled-backed multi-tree store의 `doc_id -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key를 함께 저장하고, `GET /api/documents` catalog는 same catalog key를 읽은 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_RUMDB_PATH`다.
- rumdb catalog는 append-only log 세트의 `doc_id -> persisted snapshot JSON bytes` key-value와 explicit `__catalog__` key를 함께 저장하고, `GET /api/documents` catalog는 startup log replay 뒤 재구축된 keydir를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_KV_PATH`다.
- kv catalog는 sled-backed `snapshots` bucket의 `doc_id -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 same bucket full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_KOIT_PATH`다.
- koit catalog는 structured JSON 파일의 `snapshots.<doc_id>` key-value를 저장하고, `GET /api/documents` catalog는 whole-file map load 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_LITE_DB_PATH`다.
- lite_db catalog는 LiteDb 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_LOG_KV_PATH`다.
- log_kv catalog는 append-only 단일 파일의 `snapshot:<doc_id> -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key를 저장하고, delete는 tombstone string으로 가린다. `GET /api/documents` catalog는 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=mhdb`

- 필수 env는 `SNAPSHOT_MHDB_PATH`다.
- mhdb catalog는 path prefix가 만드는 `<path>.pag`/`<path>.dir` DBM 파일 쌍에 `snapshot:<doc_id>` payload와 `__catalog__` blob을 chunked key로 나눠 저장한다. `GET /api/documents` catalog는 catalog blob 뒤 각 snapshot blob을 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=loro_kv`

- 필수 env는 `SNAPSHOT_LORO_KV_PATH`다.
- loro_kv catalog는 단일 binary SSTable 파일의 `doc_id -> persisted snapshot JSON bytes` key-value를 저장하고, save/delete마다 `MemKvStore::export_all` 결과를 temp+rename으로 확정한다. `GET /api/documents` catalog는 full scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=luckdb`

- 필수 env는 `SNAPSHOT_LUCKDB_PATH`다.
- luckdb catalog는 LuckDB `backend.snapshots` collection의 JSON document에 `doc_id`와 persisted snapshot JSON을 함께 저장하고, `GET /api/documents` catalog는 collection full query 뒤 각 payload를 복원해 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=rubin`

- 필수 env는 `SNAPSHOT_RUBIN_PATH`다.
- rubin catalog는 Rubin `MemStore` JSON 파일의 string map에 `doc_id -> persisted snapshot JSON` entry를 저장하고, `GET /api/documents` catalog는 같은 JSON 파일을 읽은 뒤 각 payload를 복원해 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=lsm_engine`

- 필수 env는 `SNAPSHOT_LSM_ENGINE_PATH`다.
- lsm_engine catalog는 WAL 파일의 `snapshot:<doc_id> -> persisted snapshot JSON string` key-value와 explicit `__catalog__` key를 저장하고, reopen 때 WAL replay로 in-memory LSM state를 재구성한다. `GET /api/documents` catalog는 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=lsm_storage_engine`

- 필수 env는 `SNAPSHOT_LSM_STORAGE_ENGINE_PATH`다.
- lsm_storage_engine catalog는 WAL/SSTable LSM 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, save/delete 뒤 flush해 `GET /api/documents` catalog가 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=lsmdb`

- 필수 env는 `SNAPSHOT_LSMDB_PATH`다.
- lsmdb catalog는 WAL/SSTable LSM 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, WAL sync-on-write 경계로 `GET /api/documents` catalog가 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=lsm_tree`

- 필수 env는 `SNAPSHOT_LSM_TREE_PATH`다.
- lsm_tree catalog는 WAL이 없는 lsm-tree primitive 디렉터리에 `snapshot:<doc_id> -> persisted snapshot JSON bytes` payload와 explicit `__catalog__` key를 저장하고, save/delete 뒤 active memtable flush로 `GET /api/documents` catalog가 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=ferrumdb`

- 필수 env는 `SNAPSHOT_FERRUMDB_PATH`다.
- ferrumdb catalog는 append-only log 파일의 `snapshot:<doc_id> -> persisted snapshot JSON` JSON value와 explicit `__catalog__` key를 저장하고, `FsyncPolicy::Always`로 write마다 sync해 `GET /api/documents` catalog가 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.

### `SNAPSHOT_STORE=mindb`

- 필수 env는 `SNAPSHOT_MINDB_PATH`다.
- mindb catalog는 WAL/SSTable LSM 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 저장하고, save/delete 뒤 `sync()`해 WAL durability 경계를 고정한다. reopen point index가 비어 있는 경우 adapter가 upstream `RecoveryManager`로 WAL을 재생하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_MMDB_PATH`다.
- mmdb catalog는 WAL/SSTable LSM 디렉터리의 `snapshot:<doc_id> -> persisted snapshot JSON` key-value와 explicit `__catalog__` key를 write batch로 함께 저장하고, sync write 뒤 flush해 `GET /api/documents` catalog가 catalog key 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_NANODB_PATH`다.
- nanodb catalog는 single JSON 파일의 root object에 `doc_id -> persisted snapshot JSON` entry를 저장하고, `GET /api/documents` catalog는 whole-file object load 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JFS_PATH`다.
- jfs catalog는 single JSON 파일의 `doc_id -> persisted snapshot JSON string` key-value를 저장하고, `GET /api/documents` catalog는 whole-file map load 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_JSON_STORE_PATH`다.
- json_store catalog는 append-only JSON line 파일의 `doc_id -> persisted snapshot` key-value를 저장하고, `GET /api/documents` catalog는 whole-file line replay와 key별 최신 offset 인덱스로 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_HMDB_PATH`다.
- hmdb catalog는 append-only schema 로그의 `doc_id -> persisted snapshot` key-value를 저장하고, `GET /api/documents` catalog는 startup log replay 뒤 메모리 map에서 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BITASK_PATH`다.
- bitask catalog는 append-only log의 `doc_id` key와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 startup log replay 뒤 재구축된 keydir를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BITKV_RS_PATH`다.
- bitkv-rs catalog는 append-only log의 `doc_id -> persisted snapshot JSON` key-value를 sync write로 저장하고, `GET /api/documents` catalog는 startup log replay 뒤 재구축된 in-memory index를 따라 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BITCASK_ENGINE_PATH`다.
- bitcask-engine catalog는 append-only bitcask-engine-rs log replay 뒤 재구축된 in-memory index에서 explicit `__catalog__` key와 `snapshot:<doc_id> -> persisted snapshot JSON` payload를 읽어 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_BLAZEUP_PATH`다.
- blazeup catalog는 blazeup `snapshots` bucket에 `snapshot:<doc_id> -> persisted snapshot JSON string` record와 explicit `__catalog__` key를 저장하고, `GET /api/documents` catalog는 catalog key 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CANDYSTORE_PATH`다.
- candystore catalog는 `doc_id` key와 explicit `__catalog__` key를 저장하고, large payload는 `set_big/get_big` 경로를 사용한 뒤 `flush`와 `checkpoint`를 거쳐 durable cursor를 전진시킨다. `GET /api/documents` catalog는 candystore keyspace에서 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CELERIX_STORE_PATH`다.
- celerix_store catalog는 Celerix Store persistence persona 파일 `snapshots.json` 안의 `documents` app map에 `doc_id -> persisted snapshot JSON` value를 저장하고, `GET /api/documents` catalog는 같은 app map을 순회해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_CAVES_PATH`다.
- caves catalog는 `<doc_id>` key마다 별도 파일에 persisted snapshot JSON bytes를 저장하고, `GET /api/documents` catalog는 directory scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_PERSISTENT_KV_PATH`다.
- persistent-kv snapshot set/WAL 디렉터리는 `doc_id -> persisted snapshot JSON bytes` key-value를 저장하고, `GET /api/documents` catalog는 same key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_NODB_PATH`다.
- 필수 env는 `SNAPSHOT_OKOFDB_PATH`다.
- okofdb catalog는 디렉터리 아래 `doc_<uuid_simple> -> persisted snapshot JSON` key-per-file 엔트리를 저장하고, `GET /api/documents` catalog는 same directory scan 뒤 각 payload를 다시 읽어 문서 메타데이터를 만든다.
- nodb keyspace는 `doc_id -> persisted snapshot` key-value를 저장하고, `GET /api/documents` catalog는 key scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- hightower-kv keyspace는 `snapshot:<doc_id> -> persisted snapshot JSON` key-value를 저장하고, `GET /api/documents` catalog는 같은 prefix scan 뒤 각 payload를 복원해 문서 메타데이터를 만든다.
- 필수 env는 `SNAPSHOT_S3_ENDPOINT`, `SNAPSHOT_S3_REGION`, `SNAPSHOT_S3_BUCKET`, `SNAPSHOT_S3_ACCESS_KEY_ID`, `SNAPSHOT_S3_SECRET_ACCESS_KEY`다.
- optional env는 `SNAPSHOT_S3_PREFIX`, `SNAPSHOT_S3_SESSION_TOKEN`, `SNAPSHOT_S3_TIMEOUT_SECS`, `SNAPSHOT_S3_PATH_STYLE`다.
- object key는 `<SNAPSHOT_S3_PREFIX><doc_id>.json` 규칙을 사용하고, `GET /api/documents` catalog는 matching object를 개별 load해 문서 메타데이터를 복원한다.

- incoming awareness JSON is validated against `AwarenessState`; malformed JSON, blank required identifiers, or invalid `user.color` values are rejected before room awareness state is updated.

- 프런트엔드는 관리 API 호출 시 `Authorization: Bearer <API_TOKEN>`을 넣어야 한다.
- 문서 생성 응답의 `credentials.access_token`을 저장하고, 같은 문서의 상세 조회, 삭제, WebSocket 연결에 재사용해야 한다.
- WebSocket 연결 경로는 문서 ID 단위로 고정하고, 브라우저 origin은 `FRONTEND_ORIGIN`과 일치해야 한다.
- 연결 후 게시하는 Yrs awareness state는 아래 구조를 표준으로 사용한다.

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

- `user.id`, `user.name`, `client.id`, `client.kind`는 trim 후 빈 문자열이면 안 된다.
- `user.color`는 `#RRGGBB` 형식의 hex color를 사용한다.
- `selection`은 선택 사항이며, 커서 위치를 보내지 않을 때는 생략할 수 있다.
- 외부 인증 연동과 사용자 프로필의 source of truth는 아직 별도 계약에 포함하지 않는다.
