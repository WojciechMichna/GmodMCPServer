AddCSLuaFile("cl_init.lua")
AddCSLuaFile("shared.lua")
include("shared.lua")

local has_module = pcall(require, "gmod_mcp_http_server")

if not has_module then
    print("[MCP Radio] WARNING: DLL module not found! The radio will not poll the HTTP server.")
end

local mySecretToken = "Bearer none"

-- pcall spróbuje wywołać include. Jeśli plik nie istnieje, nie wywali błędu gry.
local success, loadedToken = pcall(include, "mcp_password.lua")

if success and type(loadedToken) == "string" then
    print("[MCP Lua Task] ZNALEZIONO HASLO: mcp_password.lua")
    mySecretToken = loadedToken
else
    print("[MCP Lua Task] BRAK PLIKU HASLA: Uzywam domyslnego")
end

function ENT:Initialize()
    self:SetModel(self.Model) 
    
    self:PhysicsInit(SOLID_VPHYSICS)
    self:SetMoveType(MOVETYPE_VPHYSICS)
    self:SetSolid(SOLID_VPHYSICS)

    local phys = self:GetPhysicsObject()
    if IsValid(phys) then
        phys:Wake()
    end

    if has_module then
        if SetMcpToken then
            SetMcpToken(mySecretToken)
        end

        if StartMcpServer then
            StartMcpServer()
            
            local timerName = "MCP_Server_Poll_" .. self:EntIndex()
            timer.Create(timerName, 0.1, 0, function()
                if IsValid(self) then
                    self:PollMcp()
                else
                    timer.Remove(timerName)
                end
            end)
        end
    end
end

function ENT:PollMcp()
    if RustPollMcpServer then
        RustPollMcpServer()
    end
    
    if PopMcpTask then
        local taskJson = PopMcpTask()
        
        while taskJson ~= nil do
            local taskData = util.JSONToTable(taskJson)
            
            if taskData then
                -- Handle Print Task
                if taskData.TaskPrint and taskData.TaskPrint.text then
                    print("[MCP Lua Task] Printing: " .. taskData.TaskPrint.text)
                end
                
                -- Handle Spawn Task
                if taskData.TaskSpawn then
                    if taskData.TaskSpawn == "Crate" then
                        print("[MCP Lua Task] Spawning a crate!")
                        local crate = ents.Create("prop_physics")
                        if IsValid(crate) then
                            crate:SetModel("models/props_junk/wood_crate001a.mdl")
                            local spawnPos = self:GetPos() + Vector(0, 0, 50)
                            crate:SetPos(spawnPos)
                            crate:Spawn()
                        end
                    elseif taskData.TaskSpawn == "Metrocop" then
                        print("[MCP Lua Task] Spawning a Metro Cop!")
                        local npc = ents.Create("npc_metropolice")
                        if IsValid(npc) then
                            -- Inteligentne losowanie pozycji (z ominięciem środka, żeby nie zablokował radia)
                            local signX = math.random(0, 1) == 0 and -1 or 1
                            local signY = math.random(0, 1) == 0 and -1 or 1
                            local offsetX = math.random(75, 150) * signX
                            local offsetY = math.random(75, 150) * signY
                            
                            -- Dodajemy 10 do Z, żeby nie zaciął się w podłodze
                            local spawnPos = self:GetPos() + Vector(offsetX, offsetY, 10)
                            
                            npc:SetPos(spawnPos)
                            npc:Give("weapon_pistol") -- Dajemy mu broń
                            npc:Spawn()
                        end
                    end
                end
            end
            
            taskJson = PopMcpTask() 
        end
    end
end

function ENT:OnRemove()
    if has_module and StopMcpServer then
        StopMcpServer()
    end
    timer.Remove("MCP_Server_Poll_" .. self:EntIndex())
end