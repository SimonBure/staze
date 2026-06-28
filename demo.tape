# staze demo tape — render with: vhs demo.tape
# Produces demo.gif (Staz bouncing in the terminal for a few seconds).

Output demo.gif

Set FontSize 18
Set Width 1000
Set Height 640
Set Padding 20
Set Theme "Catppuccin Latte"

# --- the actual demo ---
Type "staze"
Sleep 800ms
Enter

# let Staz bounce around for a few seconds
Sleep 6s

Enter
Sleep 6s
Escape
Sleep 3s
Right
Sleep 1s
Enter
Sleep 5s
Left
Sleep 5s
Right
Sleep 0.5s
Right
Sleep 5s
Escape
