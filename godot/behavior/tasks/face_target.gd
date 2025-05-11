#*
#* face_target.gd
#* =============================================================================
#* Copyright (c) 2023-present Serhii Snitsaruk and the LimboAI contributors.
#*
#* Use of this source code is governed by an MIT-style
#* license that can be found in the LICENSE file or at
#* https://opensource.org/licenses/MIT.
#* =============================================================================
#*
@tool
extends BTAction
## Flips the agent to face the target, returning [code]SUCCESS[/code]. [br]
## Returns [code]FAILURE[/code] if [member target_var] is not a valid [Node2D] instance.

## Blackboard variable that stores our target (expecting Node2D).
@export var target_var: StringName = &"target"

# Display a customized name (requires @tool).
func _generate_name() -> String:
	return "FaceTarget " + LimboUtility.decorate_var(target_var)


# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	var target: Node2D = blackboard.get_var(target_var)
	if not is_instance_valid(target):
		return FAILURE
	var dir: float = target.global_position.x - agent.global_position.x
	agent.velocity = Vector2.ZERO
	agent.face_dir(dir)
	var self_center = Vector2(agent.global_position.x + 8, agent.global_position.y + 8)
	var target_center = Vector2(target.global_position.x + 8, target.global_position.y + 8)
	if target_center.distance_squared_to(self_center) > 840:
		return RUNNING
	return SUCCESS
