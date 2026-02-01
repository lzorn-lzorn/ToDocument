local A = {}
-- @!all 不会导出
-- @brief 这是一个示例函数      (brief)
-- @param x number 第一个参数  (Parameter: name, type_name, description)
-- @param y number 第二个参数  (Parameter: name, type_name, description)
-- @return number 返回值说明   (Parameter: "", type_name, description)
-- @description
--     \text text  (DescriptionType.Text)
--     \code{}     (DescriptionType.Code)
--     \formula{}  (DescriptionType.MathFormula)
--     \list       (DescriptionType.BulletList)
--         - item1
--         - item2
--     \html url   (DescriptionType.HTMLLink)
function add(x, y) 
end

local function add(x, y)
end

function A:add(x, y)
end

function A.add(x, y)
end
 
function A.suba() end

function A.sub1() -- end
end


function A.sub( x,
 				y,
			    z)

end
