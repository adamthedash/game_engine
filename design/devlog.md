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

Here's what today's result looks like, 2 chunks each made of a different block type.  

![](./images/day3.png)



