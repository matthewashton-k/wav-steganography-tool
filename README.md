**Sound Encoder**

This is a gui program that encodes a message into the least significant bits in a WAV file and decodes messages from WAV files.
The program also supports generating WAV files and uploading encoded files to SoundCloud.

**Features**:

 *   Encode a message into a WAV file
 *   Decode a message from a WAV file
 *   Generate a WAV file with a specified filename and filesize
 *   Upload an encoded WAV file to SoundCloud (not sure if this actually works, because I dont have a soundcloud oath token yet)


This program uses clap and klask to create the gui, and hound to generate and mess with the wav files.

What it looks like:![alt text](https://i.imgur.com/x1WWJRo.png "gui tool")



**WARNING: I was not able to get a soundcloud api key so I have no way of knowing if the soundcloud upload actually works or not until I get one **
