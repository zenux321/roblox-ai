--!strict
-- Checkpoint, respawn și finish, validate numai pe server.
-- Generat de furnizorul demonstrativ (răspuns prestabilit), nu de un model real.
local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local Config = require(ReplicatedStorage.Shared.Config)

local lastCheckpoint: { [Player]: number } = {}
local lastRequest: { [Player]: number } = {}

-- Clientul poate doar să ceară un respawn; serverul decide unde.
local respawnRequest = Instance.new("RemoteEvent")
respawnRequest.Name = "RequestRespawn"
respawnRequest.Parent = ReplicatedStorage

local function course(): Folder?
	return workspace:FindFirstChild("Course") :: Folder?
end

local function checkpointPart(index: number): BasePart?
	local folder = course()
	if not folder then
		return nil
	end
	for _, child in folder:GetChildren() do
		if child:IsA("BasePart") and child:GetAttribute("CheckpointIndex") == index then
			return child
		end
	end
	return nil
end

local function respawn(player: Player)
	local character = player.Character
	local root = character and character:FindFirstChild("HumanoidRootPart") :: BasePart?
	local index = lastCheckpoint[player]
	local target = if index then checkpointPart(index) else nil
	if root and target then
		root.CFrame = target.CFrame + Vector3.new(0, 4, 0)
	elseif character then
		player:LoadCharacter()
	end
end

local function onTouched(part: BasePart, hit: BasePart)
	local player = Players:GetPlayerFromCharacter(hit.Parent)
	if not player then
		return
	end
	local index = part:GetAttribute("CheckpointIndex")
	if typeof(index) == "number" and (lastCheckpoint[player] or 0) < index then
		lastCheckpoint[player] = index -- doar serverul scrie progresul
	elseif part:GetAttribute("Kills") then
		respawn(player)
	elseif part:GetAttribute("IsFinish") then
		player:SetAttribute("Finished", true)
	end
end

respawnRequest.OnServerEvent:Connect(function(player: Player, requested: unknown)
	-- Cererea falsificată (alt checkpoint, prea des) este ignorată.
	local now = os.clock()
	if now - (lastRequest[player] or 0) < Config.RespawnCooldownSeconds then
		return
	end
	lastRequest[player] = now
	if requested ~= nil and requested ~= lastCheckpoint[player] then
		return
	end
	respawn(player)
end)

local function watch(folder: Instance)
	for _, child in folder:GetChildren() do
		if child:IsA("BasePart") then
			child.Touched:Connect(function(hit)
				onTouched(child, hit)
			end)
		end
	end
end

local folder = course() or workspace:WaitForChild("Course")
watch(folder)

Players.PlayerRemoving:Connect(function(player)
	lastCheckpoint[player] = nil
	lastRequest[player] = nil
end)
