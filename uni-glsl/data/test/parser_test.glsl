 struct DirectionalLight { 
 vec3 direction ; 
 vec3 ambient ; 
 vec3 diffuse ; 
 vec3 specular ; 
 } ; 
 struct PointLight { 
 vec3 position ; 
 float constant ; 
 float linear ; 
 float quadratic ; 
 vec3 ambient ; 
 vec3 diffuse ; 
 vec3 specular ; 
 float rate ; 
 } ; 
 uniform vec3 uViewPos ; 
 uniform sampler2D uDiffuse ; 
 uniform float uShininess ; 
 varying vec3 vFragPos ; 
 varying vec2 vTexCoords ; 
 varying vec3 vNormal ; 
 uniform DirectionalLight uDirectionalLight ; 
 uniform PointLight uPointLights [ 4 ] ; 
 vec3 CalcDirectionalLight ( DirectionalLight light , vec3 normal , vec3 viewDir ) ; 
 vec3 CalcPointLight ( PointLight light , vec3 normal , vec3 fragPos , vec3 viewDir ) ; 
 void main ( void ) { 
 vec3 norm = normalize ( vNormal ) ; 
 vec3 viewDir = normalize ( uViewPos - vFragPos ) ; 
 vec3 result = CalcDirectionalLight ( uDirectionalLight , norm , viewDir ) ; 
 for ( int i = 0 ; i < 4 ; i ++ ) 
 result += CalcPointLight ( uPointLights [ i ] , norm , vFragPos , viewDir ) ; 
 gl_FragColor = vec4 ( result , 1.0 ) ; 
 } 
 vec3 CalcDirectionalLight ( DirectionalLight light , vec3 normal , vec3 viewDir ) 
 { 
 vec3 ambient = light . ambient * vec3 ( texture2D ( uDiffuse , vTexCoords ) ) ; 
 vec3 lightDir = normalize ( - light . direction ) ; 
 float diff = max ( dot ( normal , lightDir ) , 0.0 ) ; 
 vec3 diffuse = light . diffuse * diff * texture2D ( uDiffuse , vTexCoords ) . rgb ; 
 vec3 reflectDir = reflect ( - lightDir , normal ) ; 
 float spec = pow ( max ( dot ( viewDir , reflectDir ) , 0.0 ) , uShininess ) ; 
 vec3 specular = light . specular * spec ; 
 return ambient + diffuse + specular ; 
 } 
 vec3 CalcPointLight ( PointLight light , vec3 normal , vec3 fragPos , vec3 viewDir ) 
 { 
 vec3 lightDir = normalize ( light . position - fragPos ) ; 
 float diff = max ( dot ( normal , lightDir ) , 0.0 ) ; 
 vec3 reflectDir = reflect ( - lightDir , normal ) ; 
 float spec = pow ( max ( dot ( viewDir , reflectDir ) , 0.0 ) , uShininess ) ; 
 float distance = length ( light . position - fragPos ) ; 
 float d = ( light . constant + light . linear * distance + light . quadratic * ( distance * distance ) ) ; 
 float attenuation = 1.0 / max ( d , 0.001 ) ; 
 vec3 ambient = light . ambient * vec3 ( texture2D ( uDiffuse , vTexCoords ) ) ; 
 vec3 diffuse = light . diffuse * diff * vec3 ( texture2D ( uDiffuse , vTexCoords ) ) ; 
 vec3 specular = light . specular * spec ; 
 ambient *= attenuation ; 
 diffuse *= attenuation ; 
 specular *= attenuation ; 
 return ( ambient + diffuse + specular ) * light . rate ; 
 } 
