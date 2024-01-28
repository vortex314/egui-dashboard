Widgets Trait
=============
This trait is used to add widgets to your application.
- constructor(config:Value, parent:Widget, context:Context, cmd:Sender<PubSubCmd>, rcv:Receiver<PubSubMsg>)
- fn show(&mut self)
- destructor(  )
# Design
Application detects if config yaml has changed, if so it reloads the config tree
if successful it adapts the loop for creating the widgets.
if any error occurs during creation, the application shows the error and goes back to waiting yaml config update.
## Example
```rust
```
# PubSubSwitch
# PubSubGraph
# PubSubTable
# SubIndicator
# SubGauge
# SubButton
# SubImage
# SubVideo
# SubAudio
# PubSubSlider
# PubSubText / PubSubNumber 
