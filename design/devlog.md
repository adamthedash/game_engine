# Dev Log

## Day 1
Today's goal was to learn the basics of windowing & rendering in rust.  
I'm following along with the [Learn WGPU](https://sotrh.github.io/learn-wgpu/) and [Winit](https://docs.rs/winit/latest/winit/) tutorials.  

For my blocks, I'm rendering each face individually. Each face consists of 4 vertices in unit space, which are mapped to quick texture that I created in GIMP.  
To create a block, I'm creating 6 different instances of a face, which just transforms the face from unit-space into world space with a translation and rotation.  
So to create a simple 8x8x8 cube of blocks (which was about the limit where performance started to degrade), It's a total of 3072 faces being rendered.  

The render pipeline is pretty close to the Learn WGPU tutorial. It consists of:  
1) A handle to my GPU (or integrated graphics in the case of my laptop).  
2) A Winit window and drawable surface on it.  
3) A queue which is like a pipe to send commands to the GPU.  
4) A command encoder which translates the cross-platform commands into hardware-specific commands to be sent down the queue.  
5) A WGSL shader which runs on the GPU to do the drawing.  
6) Bind groups which allow you to group resources and swap them out easily during the rendering pass. The "layout" tells the shader code how the data buffers are interpreted, while the bind group itself links in the actual data buffers.  
7) A render pipeline, which is basically a combination of all of the above. You'd usually have one pipeline for each type of rendering you want to do. In my case a renderable in-world object.  

For the texture rendering, I've also got a depth buffer, which makes sure that objects are rendered in the right order. I don't think this prevents the GPU from actually doing the rendering, only that pixels are rendered in the right order, so it doesn't help much with performance.  

Since this is just a simple rendering MVP, I'm loading all of the data buffers onto the GPU up-front. In a real game I'll want to do this dynamically as the player moves around / interacts with the world.  
During the rendering pass which happens every frame, I first do some setup. 
1) I tell my encoder to start a new render pass, clearing the old screen and setting it to grey (my skybox colour). I also clear my depth buffer.  
2) I load up the render pipeline and bind groups for my textures and camera (will discuss camera later).  
3) I load up my data buffers. They're already on the GPU, they just need to be set as active.  
4) I send a draw command. This basically just executes the shader code on the loaded data.  

I also have some debug text that I want to draw on screen. Since I'm not using any sort of UI library, I'm just using [wgpu-text](https://github.com/Blatko1/wgpu-text).  
For now I'm just showing the camera variables, since it was a bit tricky to get the camera controller code right without visual feedback.  
The text render pass is a lot simpler, I just begin a new render pass, this time without clearing the screen, and the text.  

All of this so far has just been preparing actions for the GPU to do, it hasn't actually ran them. The last thing to do is to execute them by sending them to the queue.  
Side note: Interacting with a GPU is usually an async thing. The only async needed was to create the initial device handle, so I'm assuming that everything else is a blocking API abstracted away by WGPU. I'm using [Tokio](https://tokio.rs/) which is probably overkill, but I'm familiar with it and it might come back into play later when I want to do something like async world loading.  

Moving onto the camera then, I'm just doing a basic one that looks at the world origin and rotates around it, similar to say a 3D modelling tool.  
The state and behaviour of the camera are split out into a Camera (state) and CameraController (behaviour). This will let me swap them out easily for different types of movement, eg. walking around vs flying.  
During the rendering, the camera is turned into a [View-Projection Matrix](https://jsantell.com/model-view-projection/), which basically encodes all the transformations needed (translation, rotation, perspective) to go from world-space to camera-space into a single matrix. The cool thing about this is that you can [compose](https://en.wikipedia.org/wiki/Transformation_matrix#Composing_and_inverting_transformations) a complex chain of transformations into a single matrix up-front, then applying them to each object is constant time.  
The camera matrix is passed to the shader during the rendering pass to put objects in the right spot.  

For the camera controller, I'm using an extremely simple one. It uses WASD, space, Z to go in/out/up/down/left/right. Winit events are intercepted and passed to the controller if they're keyboard inputs.  

All in all pretty happy with day 1's progress. Got the basics of rendering and event handling sorted. Here's what the final result looks like:  
![](./images/day1.png)


## Day 2
Today I wanted to do 2 things: 1) Improve the camera 2) Work with 3D models rather than creating blocks manually from faces.  

For the camera, I wanted to change the behaviour of the controller into something more fit for a game. So I decided to go for a simple flying style (think minecraft creative mode).  
Previously, my camera state was stored as a position and a "looking at" location. This is fine for an orbiting style camera like I had before, but for a player camera I switched to a position + pitch/yaw. This was easier to work with since I'd just be treating the mouse movement as angle deltas.  
One edge case I had to deal with was when the player looks straight up/down. When the up & looking at vectors are parallel things get funky with the calculations. Instead of actually dealing with this, I decided to just clmp the viewing angles so we wouldn't ever reach this state.  
To capture the mouse movements Winit has an AxisMotion event which tells you where the mouse has been moved to on each axis. I'm just mapping that to an angle delta in the controller.  
I also added a toggle button (Esc) for the controller. This will be used down the line to disable the camera movement when the player is in an interface like they're inventory.  

For the block modelling, it's following on to the end of the Learn WGPU tutorial. I'm using the [Wavefront OBJ](https://en.wikipedia.org/wiki/Wavefront_.obj_file) format, since it's human readable and most modelling software can export to it. It's a data file with the vertices, faces, normals, and texture coordinates for particular model. In my case it's a simple cube, so I got Claude to create one for me.  
There's also an associated [Material Template Library](https://en.wikipedia.org/wiki/Wavefront_.obj_file#Material_template_library) file, which defines textures, lighting, transparency, etc. For this I just pointed at my previous smiley face image.  
Other than loading the new model file, the rest of the process was fairly straight forward. I use [tobj](https://docs.rs/tobj/3.2.5/tobj/index.html) for loading the files, and then just create the vertex & index buffers from before.  

Result for today looks pretty much the same as yesterday. There's something funky going on with the block rendering, but that's a problem for next time.  
![](./images/day2.png)


## Day 3
Today's goals:  
1) New block types  
2) A data structure for the world state, chunks, blocks, etc.  
3) Fix the rendering bug  

A voxel game is pretty boring if there's just a world full of one thing, so I wanted to have a way to easily extend to different block types. [Claude ended up helping me here](https://claude.ai/share/8320c181-b6b1-4ade-9cf5-b48b4e1399fe) by pointing towards 2D texture arrays. There are a similar enough concept to sprite sheets - a single texture that you index into.  
Extending my render pipeline to support texture arrays was trivial enough; Textures are packed into a single buffer, then indexed into by the shader by passing in the block ID as the index.  
The new block, creatively named smiley2, is just a re-coloured version of the original. I'm able to re-use the block mesh and just extend the MTL file with the new texture.  

Next it was time to tackle the world data. Since I want this to support very large (or infinite) worlds, chunking the world data was a no-brainer. This'll allow me to only load relevant parts of the world, and introduce a coarser granularity when doing various computations. I also didn't want to limit worlds to 2D, so my chunks will be cubes rather than vertically infinite chunks a-la minecraft.  
[Nice blog](https://0fps.net/2012/01/14/an-analysis-of-minecraft-like-engines/) on performance chararcteristics for voxel engines.  
When considering which data structures to choose, I'll consider a few access patterns:  
1) For the rendering pass, I'll need to visit every block so I don't want a bunch of expensive pointers or hashes.  
2) For player interactions, they'll likely be localised to around the player's position. So I'll need to be able to look up based on a position key.  
3) For block-block interactions, they'll likely be adjacent, so similar to #2  
4) Completely random access such as random events, monster spawns.  
5) Only loading needed chunks, which will be sparse.  

I decided to go with a HashMap for holding the chunks at a top level. This will allow me to have a sparse representation of loaded chunks. Hashing is pretty expensive in a hot loop, but I'm hoping I can minimise the number of chunk lookups I need to perform.  
For blocks within a chunk, I'll be storing them as a 3D array. This will allow very fast access once we're in the context of a chunk. It should also be the most compact way of storing the really granular block data.  
Here's how that'll look in terms of access:  
(C: Number of chunks, B: Number of blocks in a chunk)  
1) Iterating over all blocks: C + C * B iterations (No hash needed as we're just looping through all entries)  
2) Random access to a block: 1x hash, 1x array lookup  
3) Iterating over blocks around a given position: Small C (worst case 8 at the corner of a chunk) hashes + N array lookups depending on range  
4) Iterating over all blocks in a chunk: 1x hash + B iterations  

One variable to tweak is the size of the chunk. If it's too small, there will be too many hashes and slow things down. If it's too big, we're losing a lot of granularity for things like lazy loading. For now I'm going with a completely arbitrary 16x16x16.  

I had some WIP code from before to do loading/saving of worlds. Right now it just serialises each chunk to a binary file, then stores a world as a folder of chunk files.  

The rendering bug from last time ended up being simple. When moving from individual faces to the block mesh, I had forgot to change the instancing code. So for each block I was actually rendering 6 blocks in a 3d "+" shape.  

I also did a bunch of cleanup & refactoring, particularly with the rendering code as it was getting pretty unwieldy.  

Here's what today's result looks like, 2 chunks each made of a different block type.  

![](./images/day3.png)


## Day 4
Today I wanted to get something a bit more interesting in my world, so it was time to tackle world gen!  

I found [this great video](https://www.youtube.com/watch?v=YyVAaJqYAfE) and [blog](https://www.alanzucconi.com/2022/06/05/minecraft-world-generation/) which goes through Minecraft's world generation and how it evolved over time.  

The basis of it is a [seedable random noise generation function](https://en.wikipedia.org/wiki/Perlin_noise), which basically means a parameterised function which maps an input (in this case x,y,z coordinates) to a random value between -1 and 1. The one I'll be using (Perlin noise) has some nice properties:  
1) You can apply it at a single point. This means that I can generate any place in the world at any time.  
2) It's locally smoothe, which results in nice terrain-like outputs.  
3) There's some parameters to tweak, so you can end up with nice rolling hills or jagged mountains.  
4) It's fast!  

I've implemented this algorithm many times in the past, so I'll be grabbing an off the shelf library this time [libnoise](https://docs.rs/libnoise/latest/libnoise/index.html).  
What I will be doing myself is layering multiple different generators together. This will let me have both coarse and fine detail in the terrain.  

To decide on what kinds of blocks will be created, I'm treating the noise output as a density. Where there is low density, there will be Air (i.e. no block), at medium density there will be dirt, and high will be stone. I'll refine the rules for this down the line, but this will give me something to start with.  

After playing around with the parameters for a while, I settled on something I liked. I want to go for something that feels like a big endless cavern, with lots of tunnels and crevices.  

Going from my toy chunks before to a "real" world immediately caused some issues. Each of my chunks has 4096 blocks in them. My camera has ~100 blocks view distance, so I thought creating chunks in a 16 chunk radius would be a good idea. This brought the number of blocks being rendered from 8192 to over 134 million! Which of course ground the game to a halt.  
If I was going to work with big worlds, then it was time to take a look at rendering performance.  

I really like performance optimisations. My day job is AI engineering, so I'm used to dealing with a lot of data processing. Typically performance improvements come down to a few options:  
1) Do less of the thing  
2) Do the same thing, but in a smarter way  
3) Do the thing on better hardware  
4) Do the thing in parallel  
5) Do a different thing that solves the same problem  

In my case, the rendering loop consisted of:  
1) Iterate over all of the blocks in the world
2) Create instance data (transformation, texture index) ready to be sent to the GPU
3) Copy the data over to the GPU
4) Draw the blocks to the screen

The obvious and most simple question to ask is "Do we need to render all the blocks at once?" and of course the answer is no. So I'm going to take perf opt #1 and smash everything I can with it.  

The first thing I did was to limit the candidate blocks to ones that it would be possible for the player to see. For this I simply skipped any chunks that were out of vision distance of the player, plus an extra chunk's distance just in case. This brought me down from 32768 chunks to just 2130 (~8.7m blocks). I decided not to go down to the block level, as it would mean millions of relatively expensive distance calculations for not a lot more shaved off.  

Secondly, I skipped rendering Air blocks (duh). This is about 50% of all blocks, so it brought the number down to ~4.4m.  

Finally, the player can never see a block unless they are exposed to the air. So I added a check for any adjacent block is of type Air. If any of them were, then the block is "exposed" and is rendered. I decided to skip checks for blocks at the edge of chunks, since that would add an extra hash per-block. This meant that the outside hull of each chunk was rendered, even if the blocks might be underground.  
This further shaved down the number of blocks to ~1.7m, which was just about renderable at single digit FPS.  

This still needs to come way down if I want a 3D world, so I'll focus more on it next time.  

Here's what that looks like (Only 2D plane of chunks to show a cross-section).  
![](./images/day4.png)


## Day 5
Today's goals
1) Continue on optimising the rendering loop  
2) Some basic collision detection  
3) Lighting  
4) On demand world generation  


Starting with the performance optimisations, I first applied a little bit of #2. Previously I was iterating over all chunks in the world and checking if they were within sight of the player. This is fine for a small world, but as we grow larger it scales with the number of generated chunks. I instead changed this over to generating some candidate chunk positions given the player's position, which scales with the vision range of the player, which in this case is constant. Realistically this didn't have much effect, but it's a bit more ergonomic and it's a check I'll want to have later on I'm sure.  

My 2nd one is back to rule #1, do less. Checking whether a block is exposed or not is relatively cheap on an individual block level, but adds up when it needs to be done for every block every frame. Realistically the world is not going to change very often (only when a chunk is generated, or when a block is broken/placed), so there's a lot of benefit in caching the exposure information and only re-calculating on a change.  
I added a new 3D array on the chunk to track whether a block is exposed. Since there's no interaction with the world right now, it is calculated once when a chunk is generated. In the rendering loop we just index into that array which is lightning fast.  

3rd, another "smarter" solution. Since I'm not updating exposure information every frame, I can use a more expensive method to give a better result. Where I previously assumed that blocks on the chunk boundaries are exposed, I now actually perform the check across chunk boundaries. This again reduces the number of blocks we end up rendering, from ~1.7m down to just 260k. The increased time spent on the check up front is noticable; there is a freeze for a few hundred miliseconds whenever a new set of chunks is generated. However it's worth it as the game sits at ~40 FPS when idle / looking around.

Moving on to collision detection, I opted for a very simple Axis-Aligned Bounding Box (AABB) approach. Since the world is made from cubes, the player probably is aswell. This made collision detection very straightforward - if the bboxes intersect, there's a collision. I changed the camera controller code around a bit so it first creates a "desired" movement vector based on the player's inputs, checks for collisions around the player, and if there is it nulls the movement in that direction. Having this in the controller also means that it's only checking for collisions





