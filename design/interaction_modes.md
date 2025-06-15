# Interaction Modes
Players can be in several "modes" of interaction with the game state
1) World state: Player can walk around, interact with the environment, fight things, etc.
2) UI state: Player has their inventory open, interacting with some interface, etc.
Players can toggle between them by pressing Esc.  

## World State
In the world state, the player's cursor is non-interactive and invisible. It's instead tied to the camera controller so they can look around.  
They will be able to move around (camera controller also), and interact with their surrounding environment. Eg. mouse clicks will translate to breaking the block they are targetting, or attacking an entity infront of them.  
For UI, only the HUD will be displayed (hotbar, map, etc.)  

## UI State
In the UI state, the link between the player inputs and camera controller is disabled. The cursor is unlocked and can be used to click on various UI elements.  
A full-screen overlay will be displayed, such as the player's inventory or an interface for something they've clicked on.  

