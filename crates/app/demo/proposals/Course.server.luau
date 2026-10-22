--!strict
-- Traseul obby-ului: cinci platforme, lavă, checkpointuri și finish.
-- Generat de furnizorul demonstrativ (răspuns prestabilit), nu de un model real.
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local Config = require(ReplicatedStorage.Shared.Config)

local course = Instance.new("Folder")
course.Name = "Course"
course.Parent = workspace

local function makePart(name: string, size: Vector3, position: Vector3, color: Color3): Part
	local part = Instance.new("Part")
	part.Name = name
	part.Anchored = true
	part.Size = size
	part.Position = position
	part.Color = color
	part.Parent = course
	return part
end

local lava = makePart("Lava", Vector3.new(120, 1, 40), Vector3.new(40, -2, 0), Color3.fromRGB(255, 90, 30))
lava:SetAttribute("Kills", true)

for index = 1, Config.PlatformCount do
	local x = (index - 1) * Config.PlatformSpacing
	local platform = makePart(`Platform{index}`, Vector3.new(8, 1, 8), Vector3.new(x, 4 + index, 0), Color3.fromRGB(182, 243, 107))
	if index == 3 then
		platform:SetAttribute("CheckpointIndex", 1)
	end
end

local finish = makePart(
	"Finish",
	Vector3.new(10, 1, 10),
	Vector3.new(Config.PlatformCount * Config.PlatformSpacing, 6 + Config.PlatformCount, 0),
	Color3.fromRGB(94, 224, 138)
)
finish:SetAttribute("IsFinish", true)
