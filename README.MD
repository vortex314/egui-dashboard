# egui-dashboard
A configurable dashboard for MQTT or Redis Publish / Subscribe system.
It provides outputs like : Status, Gauge, Graphs, Progress
It provides inputs like : Slider, Button, Switch, ValueInput
## Example
```xml
<Dashboard h="768" w="1024" url="https://pcthink.local:353/ws"
    xmlns="config">
    <Row h="40">
        <Label label="Dashboard LIMERO " w="400" text_size="30" src="src/ws2redis/system/alive"/>
    </Row>
    <Row h="40">
        <Status w="100" src="src/ws2redis/system/alive" ok="true" nok="false" label="ws2redis" />
        <Status w="200" src="src/esp32/sys/alive" ok="true" nok="false" label="ESP8266" />
        <Status w="300" src="src/tiva/sys/alive" ok="true" nok="false" label="TIVA" />
    </Row>
    <Row h="150">
        <Gauge w="100" src="src/esp32/sys/heap" min="0.0" max="50000.0" label="Heap" unit="Bytes" />
        <Gauge w="200" src="src/esp32/sys/temp" min="-100" max="200" label="Temp" unit="C" />
        <Gauge w="300" src="src/esp32/sys/heap" min="0.0" max="50000.0" label="Heap" />
        <Gauge w="400" src="src/esp32/sys/heap" min="0.0" max="50000.0" label="Heap" />
    </Row>
    <Row h="30">
        <Button w="100" label="RESET" dst="dst/esp32/sys/reset" pressed="true" released="false" />
        <Button w="200" label="LED" dst="dst/esp32/sys/led" pressed="on" released="off" />
    </Row>
    <Col h="500">
        <Table h="100" w="250" src="src/esp32/*" label="ESP32" timeout="1000" />
        <Slider label="Voltage" w="500" unit="V" dst="dst/esp32/sys/voltage" min="0.0" max="5.0" step="0.1" />
        <Switch label="Power" w="500" dst="dst/esp32/sys/power" on="on" off="off" />
        <Plot label="Voltage" w="500" unit="V" src="src/esp32/sys/voltage" min="0.0" max="5.0" msec="10000" />
        <Label label="Just a label text" h="40" text_size="10"/>
        <ValueOutput label="Voltage" w="500" unit="V" prefix="$" src="src/esp32/sys/voltage" />
        <Progress label="Voltage" unit="V" w="500" h="20" src="src/esp32/sys/voltage" min="0.0" max="5.0" />
        <Map label="Location" image="img/map.png" w="500" src="src/esp32/sys/location" />
        <Compass label="direction" w="100" src="src/esp32/sys/compass" />
        <ValueInput label="Voltage" unit="V" w="500" src="src/esp32/sys/voltage" />
    </Col>
</Dashboard>
```
![prototype](./doc/prototype.png) 
