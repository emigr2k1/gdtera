tool
extends Node2D

var font = load('res://jetbrains_mono.tres')

func _draw():
	draw_char(font, Vector2(100, 100), "E", "X", Color.red)
