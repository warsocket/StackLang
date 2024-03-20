#!./target/release/stacklang

# stack 0 contains [1], stack 1 contains [2]
(1)((:
(0

# Write stack 0 value
((^)		# Clear reg to 0
(:			# Goto stack 0
(110000		# ascii "0"
+			# add ascii "0" (so we can write value from 0-9)
(1)			# reg to 1
$			# WRITE (reg was still 1)

# Write stack 1 value
(:			# goto stack 1 (reg is still 1)
((^)		# Clear reg to 0
(110000		# ascii "0"
+			# add ascii "0" (so we can write value from 0-9)
(1)			# reg to 1
$			# WRITE (reg was still 1)
