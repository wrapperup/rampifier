<h1 align=center>Plate Rampifier</h1>

<p align=center>
<img src=https://user-images.githubusercontent.com/7478134/148273037-be45d3f8-75d2-4a96-9b5c-eeae76c170ea.png>
</p>
<br>

This is a sample tool created that takes plates and rampifies it. May be super useful for creating (organic) brick props like trees, rocks, terrain, etc.

## Using Plate Rampifier
[Download Plate Rampifier from here](https://github.com/Wrapperup/rampifier/releases)

### Preparing in.brs
Simply make sure your build is plate aligned. Plates can be resized, and you can use microbricks as long as they are plate sized and plate shaped (so may as well use plates!) Ensure it is aligned to Plate's grid for best results.

### Generating out.brs
Rampifier takes two arguments, the input of the save file and the output `.brs` path. 

For example, use
`plate-rampifier the_output.brs my_input.brs` or any path to rampify a save. If either are not specified, `in.brs` and `out.brs` are used in the same directory as the binary file.
