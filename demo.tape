# staze demo tape — render with: vhs demo.tape
# Produces demo.gif (the home galaxy, a session starfield, then history).

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

# let the galaxy turn for a few seconds
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
