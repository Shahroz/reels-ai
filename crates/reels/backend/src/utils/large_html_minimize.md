Sometimes we save too big styles, we should have a method to compact it - reduce redundancies there are still some inline base64 images

<image xlink:href="data:image/png;base64,12312312312312312123123123" style="background-color: rgba(0, 0, 0, 0); color: rgb(180, 161, 114);"></image>
The final style should not be bigger than 100k tokens
so we need to progressively minimize large inline strings, repeated strings, inline images, svg paths etc.
the methods to minimize the style should start from the least destructive to the most desctructive even removing some html fragments that seem duplicated.

Probably we need to do it in html space by looking at single tags and their text representation if the tag is more than 500 characters remove it.

Also if we are sure that those are assets like inline svg paths we could try to export them as assets as it is done with some html tags.

todo:
- remove comments from html
- <image xlink:href="data:image/png;base64,basdfasdfasdfasdfadsf"...> is not extracted as assets
- remove <style> tags and their contents (the styles are all inline)
- remove big svg path (bigger than some threshold?) - we want to steel preserve illustrations etc but remove unreasonably large svg - or maybe it is better to keep small svg inline and save as assets larger ones - yes I think that's better
<path fill-rule="evenodd" clip-rule="evenodd" d="M65.3675 16.3675C65.9075 16.592 66.4567 16.7043 67.015 16.7043C67.9373 16.7043 68.7626 16.4798 69.4908 16.0307C70.2189 15.5817 70.7833 14.9628 71.1838 14.1739C71.5843 13.379 71.7845 12.4778 71.7845 11.4705C71.7845 10.4632 71.5782 9.56211 71.1656 8.76718C70.7529 7.97226 70.1795 7.35634 69.4452 6.91943C68.711 6.47646 67.8766 6.25801 66.9421 6.26407C66.3475 6.26407 65.7771 6.37937 65.2309 6.60996C64.6848 6.84055 64.2297 7.16823 63.8656 7.593C63.8375 7.62534 63.8102 7.658 63.7837 7.69099V2.2591H61.2897V16.5132H63.7655L63.7605 15.1816C63.8245 15.2601 63.893 15.3368 63.9657 15.4118C64.3601 15.8184 64.8274 16.1369 65.3675 16.3675ZM67.9161 14.0647C67.5095 14.3135 67.0453 14.4379 66.5234 14.4379C66.0076 14.4379 65.5343 14.3104 65.1035 14.0556C64.6726 13.7946 64.3298 13.4397 64.0749 12.9906C63.8261 12.5416 63.7017 12.0318 63.7017 11.4614C63.6957 10.891 63.817 10.3813 64.0658 9.93227C64.3207 9.47716 64.6635 9.1252 65.0944 8.87641C65.5252 8.62155 66.0016 8.49715 66.5234 8.50322C67.0453 8.49715 67.5095 8.61851 67.9161 8.86731C68.3287 9.11003 68.6442 9.45895 68.8627 9.91406C69.0872 10.3631 69.1995 10.8789 69.1995 11.4614C69.1995 12.044 69.0872 12.5598 68.8627 13.0088C68.6442 13.4579 68.3287 13.8098 67.9161 14.0647Z" fill="currentColor" style="background-color: rgba(0, 0, 0, 0); color: rgb(8, 8, 8);"></path>
- remove iframes
- remove redundant html tags progressively until reaching 100k tokens ~ 300k characters