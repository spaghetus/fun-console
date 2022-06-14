* = $0200
.text "#!fun"
.byte $0A
.text "COOL GAME "
.fill 240, [$00]
entry
	; Set bank 7
	lda #0
	sta $4018
	; Increment the run count
	inc $3C00
	jsr print1
	jsr printn
	jsr print2
	jmp finish
text1
	.null "This game has been run "
text2
	.text " times."
	.byte $0A
	.byte $00
print1
	ldx #0
	jmp loop1
loop1
	lda text1,x
	inx
	sta $4019
	adc #0
	bne loop1
	rts
print2
	ldx #0
	jmp loop2
loop2
	lda text2,x
	inx
	sta $4019
	adc #0
	bne loop2
	rts

printn
	lda $3C00
	adc #'0'
	sta $4019
	rts

finish
	jmp finish
