"""Capture current metadata using the prior checkpoint's immutable format."""
from pathlib import Path

source = (Path(__file__).parent / 'checkpoint-audits-179.py').read_text(encoding='utf-8')
source = source.replace('179', '182').replace('112', '113')
source = source.replace('bounded269roles', 'bounded275roles')
source = source.replace('CameraREG033-036 exact source bodies to fresh Root03 admission', 'None through Camera037/Rig035/Native034/Root017 under scope053; recheck any later registrations')
source = source.replace('NativeREG034 existing P16 Milliliters74-106', 'Native034 published in053 after actual03 proof180')
source = source.replace('Global0522820+236 through Camera032/Rig035/Native033/Root017; pending Camera033-036 and Native034 registered before actual Root03 admission. All source/proof qualifications retained.', 'Global0532825+237 through Camera037/Rig035/Native034/Root017 after actual03 proof180. All source/proof qualifications retained; later registrations require fresh admission.')
source = source.replace('Fresh Camera033-036/Native034 admission and further04 source comparison next.', 'Fresh actual03 proof180 and scope053 published; Root04 full motor/config/plugin/decision operations113 now pinned. Continue remaining04 then08.')
source = source.replace('Root04 records101-113, recovered103 under170/104, new fresh03 scope050-052', 'Root04 records101-113, recovered103 under170/104, new fresh03 scope050-053')
exec(compile(source, str(Path(__file__)), 'exec'))
