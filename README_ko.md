# 범용 월패드 시리얼 통신 MQTT 브릿지 (브로커 포함) 

[한국어](README_ko.md) | [English](README.md)

이 프로젝트는 RS485-MQTT 브로커입니다. 스마트홈을 구축하려면 이 앱을 다른 앱과 함께 사용할 수 있습니다.

HomeAssistant를 사용 중 이라면 [HomeAssistant 연결 가이드 (준비중)]()를 참고하세요.

다른 MQTT 클라이언트를 사용한다면 [MQTT 장치 토픽 스펙 (영문)]를 참고하세요.

## 시작하기

자신의 환경에 맞는 빌드를 [다운로드](https://github.com/blkis0/universal-wallpad-bridge/releases) 하세요.

```rumqttd.toml``` 파일을 [예시](https://github.com/blkis0/universal-wallpad-bridge/rumqttd.toml)를 참고해 작성해주세요.
```--rumqttd=<path>``` 옵션을 사용해 다른 경로에 파일을 저장할 수 있습니다.

이후, 터미널을 실행해 아래 명령어를 입력하면 시작할 수 있습니다.

Windows
```
./universal-wallpad-bridge.exe -m <제조사> -- <주 시리얼 포트 (예: COM1)>
```

Linux
```
./universal-wallpad-bridge -m <제조사> -- <주 시리얼 포트 (예: /dev/ttyUSB0)> 
```

### 사용 가능한 옵션

|필수 / 선택|키|값: 타입|설명|
|---|---|---|---|
|필수||<시리얼 포트: string>|주 시리얼 포트 - 모든 기기가 연결된 포트 입니다. (예: COM1, /dev/ttyUSB0, 등...)
|필수|-m / --manufacturer|[제조사: string]|월패드 제조사 - 해당 아파트의 [월패드 제조사](#월패드-제조사-별-기능)를 지정합니다.
|선택|-f / --features|[기기1: string] [기기2: string]...|사용 가능 기기 - 해당 어파트에서 [사용 가능한 기기](#테스트된-아파트-목록)를 지정합니다. (쉼표로 구분)
|선택|-s / --second-port|<시리얼 포트: string>|보조 시리얼 포트 - 디지털 도어락과 계량기가 연결된 포트 입니다. (예: COM2, /dev/ttyUSB1, 등...)
|선택|-r / --rumqttd|<경로: string>|MQTT Broker 설정 - rumqttd 설정 파일의 경로를 지정할 수 있습니다. (기본값: ./rumqttd.toml )
|선택|-t / -pariod|<숫자: uint64>|페킷 조회 간격 - 기기의 상태를 조회하는 시간을 설정 수 있습니다. (기본값: 2초)
|선택|--log|<경로: string>|오가는 페킷 정보를 파일로 저장합니다.
|선택|-v||모든 페킷의 상세 정보를 확인 할 수 있습니다.

### 더 알아보기

- [MQTT 장치 토픽 스펙 (영문)]
- [rumqttd 저장소 (Github)](https://github.com/bytebeamio/rumqtt/tree/main/rumqttd)
- [앱을 Docker에서 사용하는 방법 (준비중)]()

## 빌드

시작하려면 먼저 [Rust 컴파일러](https://www.rust-lang.org/tools/install)가 시스템에 설치되어 있어야 합니다.

개발 환경이 구성 되었다면 다음 차례를 따라해주세요.

- 프로젝트를 다운로드 받아 압축을 해제하거나, 아래 명령으로 저장소를 복제 해주세요.
    ```bash
    git clone https://github.com/blkis0/universal-wallpad-bridge
    ```

- 프로젝트 폴더로 이동해 아래 명령으로 빌드를 시작할 수 있습니다.
    ```bash
    cargo build --release
    ```

- 이제 target 폴더 내에서 프로그램을 찾을 수 있습니다.

<br>

혹은, 프로그램을 즉시 실행하고 싶다면 다음 명령어를 사용할 수 있습니다.

```bash
cargo run --release -- -m <제조사> -- <주 시리얼 포트 (예: COM1 또는 /dev/ttyUSB0)>
```

## 호환성

현재 일부 월패드에서만 사용이 가능합니다. 세대내 월패드 제조사가 아래 목록에 있는지 확인해주세요.

Ctrl + F 또는 F3을 사용해 제조사를 검색할 수 있습니다.

### 사용 가능한 제조사

- 현대통신 (hyundai_ht)

### 월패드 제조사 별 기능

#### 현대통신 (hyundai_ht)

|구현 여부|영문명|이름|타입|설명
|---|---|---|---|---|
O|floor_heating|바닥 난방|-|최대 4개의 방 (거실+방3) 까지 테스트 완료
O|ventilator|환기|-|신우공조 기기 바이패스 사용 가능
O|living_room_lights|거실등|-|-
O|realtime_energy_meter|실시간 에너지 사용량|자동 조회 센서|-
-|gas_valve|가스 벨브|-|개발 중
-|elevator_call|엘리베이터 호출|-|개발 중
-|lights|일괄 소등|-|개발 중
-|doorlock|디지털 도어락|-|개발 중
<br>


[MQTT 장치 토픽 스펙 (영문)]: https://github.com/blkis0/universal-wallpad-bridge/wiki/MQTT-Device-Topic-Specification