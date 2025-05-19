# Drop
## Creator: Sarah Dylan
## What I did

I started by tackling the magnetometer. The [example code](https://github.com/nrf-rs/microbit/blob/main/examples/magnetometer/src/main.rs) from the microbit v2 crate's example folder was really helpful in setting this up. I started by just outputting the x, y, and z like we did in class. Then I moved on to the acceleration. I originally was having a hard time getting the 0.25 value to work so I thought that maybe adding a sqrt function would help. I found a crate to do that and then change the reading to < 0.5, but this made no difference. When I was testing the code I bumped it up to 0.7 and it worked fine, but I changed it back to 0.5 because that was part of the requirements. 

Once I was able to detect when the board was falling, I moved on to the non-blocking display. The example from class was more than enough to get this done quickly.

The last part and the part that I wasted most of my time on was the sound. I tried really hard to make it a separate interrupt that would start when the acceleration exceeded the threshold and then stop when it stopped falling. However, this was kind of impossible for me to figure out. I ended up realizing that I could just loop it for some arbitrary amount of times because otherwise the speaker would just click on and off over and over again. So I copied the loop from the example code and just ran it for like 200 loops that would restart if it was still falling. This worked out fine.

## How it went

I really hated the sound part of this project. It made sense in my head for it to be like an async task or a separate thread that the display or main thread could start and stop. Looping for x times to produce a sound seemed kind of cheating, but it works and I am satisfied with it. 

It was really hard to test this as well as I broke my battery thing dropping it so many times, and then I did not have a wire that would let me drop it far. Also, I kept having to adjust the delay at the end of the main loop to make it fast enough to detect the falling but slow enough to not constantly refresh. 