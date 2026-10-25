# matcha AI 공통 조작서

matcha는 AI가 게임보이·게임보이 컬러 ROM을 실행하고, 화면을 확인하고,
버튼을 누르고, CPU·메모리·그래픽을 조사하는 에뮬레이터다.
특정 AI 모델에 종속되지 않는다. 사용하는 앱이 로컬 MCP 서버와 도구 호출을
지원해야 하며, 화면을 보고 판단하려면 이미지 결과를 모델에 전달할 수 있어야 한다.
연결 방법은 [README.md](README.md)에 있다.

## 1. 연결과 실행 단위

- 전송 방식: 표준 입출력(`stdio`)을 사용하는 줄 단위 JSON-RPC 2.0.
- 서버 실행: 패키지에서 `node server/server.mjs`. 보통 AI 앱이 실행과 종료를 맡는다.
- 호환 프로토콜: `2024-11-05`, `2025-03-26`, `2025-06-18`, `2025-11-25`.
- 연결 순서: `initialize` → `notifications/initialized` → `tools/list` → `tools/call`.
  일반 MCP 앱은 이 순서를 대신 처리한다.
- 서버 프로세스 하나에 게임 한 개가 로드된다. 세션을 분리하려면 서버도 따로 실행한다.
- 게임 시간은 `load_rom`, `press`, `run`, `step` 등의 실행 도구를 호출할 때 진행된다.
  `screenshot`, `state`, `read_memory`로 관찰하는 동안 게임이 자동으로 진행되지는 않는다.
- 약 60프레임이 게임 시간 1초다. 실제 대기 시간과 구분해서 기록한다.
- 도구의 `content`에는 텍스트와 PNG 이미지가 들어온다. `isError: true`이면
  성공한 것으로 처리하지 말고 오류 메시지에 따라 인자나 실행 순서를 수정한다.
- `tools/list`가 반환하는 실제 스키마를 우선한다. 아래 예시는 도구 이름과 인자를
  설명하며, 앱이 이름 앞에 서버 접두사를 붙이는 경우 해당 이름을 사용한다.

직접 MCP 클라이언트를 구현한다면 도구 호출은 다음 형태다.

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"load_rom","arguments":{"sample":"libbet","model":"auto","frames":60}}}
```

## 2. 처음 게임을 실행할 때

1. 사용자 ROM이 있으면 절대 경로로 불러온다.
   `load_rom({"path":"/absolute/path/game.gb","model":"auto","frames":60})`
2. 샘플을 시험할 때는 `load_rom({"sample":"libbet","frames":60})`을 호출한다.
   샘플 이름은 `tobu`, `libbet`, `shock-lobster`, `2048`이다.
3. 반환된 제목, DMG/CGB 모델, 실행 상태와 이미지를 확인한다. 화면에서 관찰한
   사실을 기록하고 게임 이름만으로 진행 상황을 추측하지 않는다.
4. 도입 화면이 계속되면 `run({"frames":120})`으로 조금씩 진행하며 확인한다.
   시작 메뉴가 보이면 `press({"buttons":["start"],"hold_frames":6,"wait_frames":60})`으로 시작한다.
5. 화면을 다시 확인하고 다음 입력을 정한다. 시작 화면 진입과 실제 플레이 성공은
   별도로 기록한다.

`model: "auto"`는 컬러 지원 헤더가 있으면 CGB, 그렇지 않으면 DMG를 선택한다.
명시적 `dmg` / `cgb` 선택은 비교 시험에 쓴다. 강제 CGB는 네이티브 컬러 모드이며
DMG 게임을 컬러로 꾸미는 호환 모드는 구현돼 있지 않다.
입력 ROM 크기는 최대 8 MiB이며, ROM-only와 MBC1/1M/2/3(+RTC)/5 계열을 지원한다.
다른 카트리지 칩이나 일부 하드웨어 타이밍을 요구하는 게임은 실행되지 않거나
다르게 동작할 수 있다.

## 3. 조작과 저장

버튼은 `a`, `b`, `start`, `select`, `up`, `down`, `left`, `right`다.
동시에 누르려면 `{"buttons":["right","a"]}`처럼 배열로 전달한다.

| 작업 | 호출 예시 | 동작 |
|---|---|---|
| 짧게 누르기 | `press({"buttons":["a"]})` | 기본 6프레임 누른 뒤 떼고 30프레임 진행 |
| 이동하며 점프 | `press({"buttons":["right","a"],"hold_frames":30,"wait_frames":10})` | 지정 시간 동안 동시 입력 |
| 기다리기 | `run({"frames":120})` | 입력 없이 약 2초 진행 |
| 계속 이동 | `run({"frames":60,"buttons":["right"]})` | 지정 프레임 동안 입력 유지 후 해제 |
| 현재 화면 보기 | `screenshot({"scale":3})` | 진행 없이 현재 화면 PNG 반환 |
| 실험 전 저장 | `save_state({"slot":"before-test"})` | 기계 상태를 메모리에 저장 |
| 실험 되돌리기 | `load_state({"slot":"before-test"})` | 저장한 상태로 복원하고 이미지 반환 |

`run`과 `press`는 브레이크포인트에서 일찍 멈출 수 있으므로 반환된 정지 사유를
확인한다. 중간 이미지가 필요 없으면 `screenshot: false`를 사용하고 마지막에
한 번 관찰한다. 같은 상태와 입력은 같은 결과를 만든다.

저장 슬롯은 **현재 MCP 서버의 메모리에만 존재**한다. `load_rom`으로 게임을 새로
불러오면 슬롯과 검색·브레이크포인트가 초기화되고, 서버를 종료해도 슬롯은 사라진다.
`reset`은 카트리지 저장 RAM, 브레이크포인트, 저장 슬롯을 유지한다. 따라서 시험
세션 전체를 폐기할 때는 서버 프로세스를 종료해야 한다. MCP 저장 슬롯과 웹
플레이어의 브라우저 저장 기능은 별개다.

## 4. 도구 17개

| 도구 | 용도와 주요 인자 |
|---|---|
| `load_rom` | 게임 로드. `path` 또는 `sample`, `model`, 초기 `frames`, `ram`, `seed` |
| `press` | 버튼 누르기·해제. `buttons`, `hold_frames`, `wait_frames`, `screenshot` |
| `run` | 프레임 진행. `frames`, 선택적 `buttons`, `screenshot` |
| `screenshot` | 현재 화면. `scale` 1–6, DMG `palette`: `grey`, `dmg`, `matcha` |
| `state` | CPU 레지스터, 인터럽트, 화면 장치 상태, 뱅크, 시간 확인 |
| `read_memory` | `address`부터 `length` 바이트 읽기. 부작용 없는 관찰 |
| `write_memory` | `address`에 `bytes` 배열 또는 16진수 문자열 쓰기 |
| `disassemble` | `address`부터 `count`개 명령 해석. 생략 시 현재 PC |
| `breakpoint` | `action`: `add`, `remove`, `list`, `clear`. `kind`: `exec`, `read`, `write` |
| `step` | `count`번 명령·인터럽트·대기 단계 실행 |
| `save_state` | 이름이 `slot`인 메모리 슬롯에 상태 저장 |
| `load_state` | 이름이 `slot`인 상태 복원 |
| `find_value` | RAM 검색. `action`: `start`, `filter`; `op`: `eq`, `ne`, `changed`, `same`, `up`, `down` |
| `serial_output` | 게임이 시리얼 포트로 출력한 텍스트 확인 |
| `profile` | `action: "start"`로 집계 시작, `"report"`로 CPU 사용·명령 통계 확인 |
| `tiles` | 현재 선택된 VRAM 뱅크의 타일 이미지. `scale` 1–4 |
| `reset` | 전원 재시작. 선택적 `ram: "zero"` 또는 `"noise"`, noise용 `seed` |

주소 문자열은 16진수로 해석한다. `"C000"`, `"0xC000"`, `"$C000"`은 같고,
숫자 `49152`도 같은 주소다. `"10"`은 16진수 0x10이다.

## 5. 내부 조사 절차

### 멈춤·깨진 화면 조사

1. `save_state({"slot":"before-debug"})`로 조사 시작점을 보존한다.
2. `state({})`로 CPU가 실행·대기·잠금 중인지, LCD 상태가 어떤지 확인한다.
3. `disassemble({"count":12})`로 현재 PC 주변 코드를 읽는다.
4. `read_memory({"address":"C000","length":64})` 또는 관련 I/O 주소를 조사한다.
5. 화면 문제는 `tiles({"scale":2})`로 그래픽 데이터가 VRAM에 도착했는지 확인한다.
6. 원인을 좁히면 실행 중단점 또는 읽기·쓰기 감시점을 설정한다.
   `breakpoint({"action":"add","kind":"write","address":"C0A2"})`
7. `run`으로 해당 접근까지 진행한 뒤 정지 보고의 PC와 `step({"count":1})`을 확인한다.
8. `breakpoint({"action":"clear"})`로 조사 설정을 정리하고 저장 상태를 복원한다.

ROM 문제와 에뮬레이터의 호환성 한계를 구분한다. CPU 잠금이나 화면 차이만으로
게임 자체의 버그라고 단정하지 않는다. 세부 절차는 `skills/debug-gameboy-rom/`의
문서와 하드웨어 참고 자료에 있다.

### 체력·점수·좌표 찾기

1. 값이 보이는 시점에서 저장하고 `find_value({"action":"start"})`를 호출한다.
2. 게임을 조작해 값이 바뀌게 한다.
3. 알려진 값이면 `find_value({"action":"filter","op":"eq","value":3})`,
   감소만 알면 `{"action":"filter","op":"down"}`으로 좁힌다.
4. 값이 그대로인 동안 `"same"` 필터로 타이머 등의 후보를 제거한다.
5. 후보 주소를 읽어 실제 게임 값과 같이 움직이는지 확인한다.
6. 값을 바꿔 확인할 때는 저장 후 `write_memory`를 사용하고, 실험 뒤 복원한다.

메모리 값은 화면 숫자와 다르게 인코딩될 수 있다. 발견한 주소는 게임·ROM 버전·뱅크와
함께 기록하며, 코드의 이름이나 원본 소스가 복구된 것처럼 표현하지 않는다.

## 6. 시험하고 기록만 남기기

시험은 사용자 플레이 세션과 분리된 **임시 폴더와 새 서버 프로세스**에서 진행한다.

1. 패키지를 임시 폴더에 풀고 그 안의 `configure.mjs`로 서버 경로를 구한다.
2. 연결 협상과 `tools/list`를 확인한 뒤 샘플 또는 복사한 시험 ROM을 실행한다.
3. 입력·화면·메모리·저장 복원 등 목적에 맞는 항목을 검증한다.
4. 날짜, 환경, 패키지 SHA-256, ROM 식별자, 모델, 입력 순서, 예상과 실제 결과,
   통과·실패·미검증 항목을 텍스트 보고서에 남긴다.
5. 성공·실패와 관계없이 `finally` 또는 종료 트랩에서 서버의 stdin을 닫고 종료를
   기다린다. 종료하지 않으면 시험에서 생성한 프로세스만 종료한다.
6. 임시 클라이언트 코드, 추출 폴더, 복사·변형한 ROM, 이미지, 상태 파일, 임시 로그를
   삭제한다. 사용자 원본 ROM과 기존 저장 데이터, 배포용 ZIP은 보존한다.
7. 임시 폴더와 시험 프로세스가 남지 않았는지 확인하고 보고서에 정리 결과를 추가한다.

자동화한 시험은 실행 제한 시간을 두고, 중간 실패에도 정리가 수행되게 만든다.
게임 한 개의 부팅 성공, MCP 프로토콜 시험, 특정 AI 앱에서의 실제 연결 성공은
각각 다른 검증이다. 시험하지 않은 AI 앱이나 전체 게임 호환성을 통과로 적지 않는다.

## 7. 연결 문제 해결

- 서버가 바로 종료되면 Node 경로, 서버 경로, 함께 배포된 `matcha.wasm`과
  `matcha.js`, `package.json`이 있는지 확인한다.
- 터미널에서 서버가 조용히 대기하는 것은 정상이다. stdio 서버는 MCP 요청을 기다린다.
- 도구는 보이지만 화면이 안 보이면 호스트의 MCP 이미지 전달과 모델 이미지 지원을 확인한다.
- 원격 서버 URL만 받는 앱에는 이 로컬 패키지의 파일 경로를 URL처럼 넣을 수 없다.
- ROM 파일 오류는 서버가 실행되는 컴퓨터의 절대 경로를 사용해 확인한다.
- 프로토콜 협상 결과를 앱이 지원하지 않으면 호환 버전을 지원하는 클라이언트가 필요하다.
