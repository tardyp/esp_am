A simple experiment to build a Radio AM emiter using ESP.

This uses the PWM to generate the base signal and the modulation is done by changing the duty cycle of the PWM.
This works approximately, and is enough to crate a wire detector for a mower robot.
Unconnect the wire from the robot base, and connect it to the ESP32, GPIO2. Use a cheap AM receptor to detect the signal it is detectable at about 4cm from the wire.