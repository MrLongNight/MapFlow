# Audio Setup Guide

VjMapper relies on the operating system's audio infrastructure to capture audio for its audio-reactive features. This guide provides instructions for setting up audio on Windows and Linux.

## Windows

On Windows, VjMapper uses the **WASAPI** backend. To capture desktop audio (i.e., what you hear), you need to enable an input device called **"Stereo Mix"** or **"What U Hear"**.

1.  **Open Sound Settings:** Right-click the speaker icon in your system tray and select "Sounds".
2.  **Go to the Recording Tab:** In the Sound control panel, click on the "Recording" tab.
3.  **Show Disabled Devices:** Right-click in the empty space in the list of devices and make sure "Show Disabled Devices" and "Show Disconnected Devices" are checked.
4.  **Enable Stereo Mix:** You should see a device named "Stereo Mix", "Wave Out Mix", "Mono Mix", or something similar. Right-click on it and select **"Enable"**.
5.  **Set as Default (Optional but Recommended):** Right-click on "Stereo Mix" again and select **"Set as Default Device"**. This will make it the default input for most applications.

Once enabled, you should be able to select "Stereo Mix" as your audio input device within VjMapper.

## Linux

On Linux, VjMapper uses the **ALSA** or **PulseAudio** backends. Capturing desktop audio is typically done using a **monitor** device in PulseAudio.

### PulseAudio Setup

1.  **List Audio Sources:** Open a terminal and run the following command to find the name of your monitor source:
    ```bash
    pactl list sources | grep 'Monitor'
    ```
    Look for a line that says `Name:` followed by something like `alsa_output.pci-0000_00_1f.3.analog-stereo.monitor`. This is the name of your monitor device.

2.  **Select in VjMapper:** In VjMapper's audio settings, select the monitor device you found in the previous step.

### PipeWire Setup

If you are using PipeWire, the setup is similar to PulseAudio. PipeWire provides a PulseAudio compatibility layer, so the same `pactl` command should work.

### JACK Setup

For professional audio users, JACK provides a powerful and flexible audio routing system. You can use a tool like **Catia** or **qjackctl** to route audio from any application to VjMapper's input.

1.  **Start JACK:** Make sure your JACK server is running.
2.  **Route Audio:** Open your JACK patchbay tool. You will see a list of audio sources and sinks.
3.  **Connect:** Connect the output of the application you want to capture (e.g., your music player) to the input of VjMapper.
