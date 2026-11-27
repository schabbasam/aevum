
# this is just my lil scratchpad where i dump my todos and known problems

## problems
- animations are a little ungood (look at the comment in task-panel.slint); low priority
- scrollbar
    - border for the scrollbar in the weekview on the left
    - no gap between it and window edge (!)
    - it is fine as is in the TaskPanel

## future features
- add animation for the collapsable TaskAttributes part
- and additionaly make the little triangle spin 180 degrees when animating
- Category
    - make the popup look decent
    - add a little icon selector and save it (kinda big addition, will take some time),
    i mean a little icon rectangle and when you click on it a popup menu appears where you can search
    a selection of icons and select it, the icons are monocolered and adjust to the background of the category/task
    and make the logic so that if it just uses all the icons with their names from a fixed directory, for example
    /ui/assets/icons/
    - dropdown selection menu for the category, for that also create a custom perfectly fitting widget like the
    ones in ui/widgets, and make them toggle through the categories with mouse wheel
- Color
    - there should be a better color picker, for example a popup with sliders for r, g, b
    - it should change automatically to the color of the category and stay locked, if a category is selected
    - a little locked icon with a lock should show if the color is unchangable because there is a category selected
- custom checkbox just like the other widgets
- edit mode
    - should load the attributes of a task if clicked on
    - uncollapse the attributes panel if it isnt already
    - all the rust logic
- duration
    - increments should be standardized and fixed in a variable, standard should be 10min
    - the default duration should be the text set in the duration input spinbox by default
- a little button in the top bar that reorganizes the TaskPanel columns to use the least height (distributes evenly)
- custom slider widget and make the zoom slider more thought through (max is so the whole timeaxis fits perfectly into the window)
- put the timer and stuff that are currently in lib.rs into a fitting place, that was just temporary

## notes
- somehow the text is ok now, it isnt blurry
- the duration-step should be updated like so:
    - just like the max length values, it should be synced with the rust backend
    - it is the same as resize-step (or should it be different?)
