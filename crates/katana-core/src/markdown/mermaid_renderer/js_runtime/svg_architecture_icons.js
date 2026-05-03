function katanaInsertArchitectureServiceIcons(svg) {
  return svg.replace(
    /(<g id="([^"]+)-service-([^"]+)" class="architecture-service"[^>]*>)([\s\S]*?)(<g><g><\/g><\/g>)(<\/g>)/g,
    (_match, start, _prefix, serviceId, body, _emptyIcon, end) =>
      `${start}${body}<g><g>${katanaArchitectureServiceIcon(serviceId)}</g></g>${end}`,
  );
}

function katanaInsertArchitectureGroupIcons(svg) {
  return svg.replace(
    /(<g transform="translate\([^"]+\)">)<g><\/g><\/g>/g,
    (_match, start) => `${start}<g>${katanaArchitectureCloudIcon()}</g></g>`,
  );
}

function katanaArchitectureServiceIcon(serviceId) {
  return serviceId === "svg"
    ? katanaArchitectureDatabaseIcon(80)
    : katanaArchitectureServerIcon(80);
}

function katanaArchitectureServerIcon(size) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 80 80"><g><rect width="80" height="80" style="fill: #087ebf; stroke-width: 0px;"></rect><rect x="17.5" y="17.5" width="45" height="45" rx="2" ry="2" style="fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;"></rect><line x1="17.5" y1="32.5" x2="62.5" y2="32.5" style="fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;"></line><line x1="17.5" y1="47.5" x2="62.5" y2="47.5" style="fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;"></line><circle cx="22.5" cy="25" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="27.5" cy="25" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="32.5" cy="25" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="22.5" cy="40" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="27.5" cy="40" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="32.5" cy="40" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="22.5" cy="55" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="27.5" cy="55" r=".75" style="fill: #fff; stroke: #fff;"></circle><circle cx="32.5" cy="55" r=".75" style="fill: #fff; stroke: #fff;"></circle></g></svg>`;
}

function katanaArchitectureDatabaseIcon(size) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 80 80"><g><rect width="80" height="80" style="fill: #087ebf; stroke-width: 0px;"></rect><ellipse cx="40" cy="22.14" rx="20" ry="7.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></ellipse><path d="m20,34.05c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></path><path d="m20,45.95c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></path><path d="m20,57.86c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></path><line x1="20" y1="57.86" x2="20" y2="22.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></line><line x1="60" y1="57.86" x2="60" y2="22.14" style="fill: none; stroke: #fff; stroke-width: 2px;"></line></g></svg>`;
}

function katanaArchitectureCloudIcon() {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 80 80"><g><rect width="80" height="80" style="fill: #087ebf; stroke-width: 0px;"></rect><path d="m65,47.5c0,2.76-2.24,5-5,5H20c-2.76,0-5-2.24-5-5,0-1.87,1.03-3.51,2.56-4.36-.04-.21-.06-.42-.06-.64,0-2.6,2.48-4.74,5.65-4.97,1.65-4.51,6.34-7.76,11.85-7.76.86,0,1.69.08,2.5.23,2.09-1.57,4.69-2.5,7.5-2.5,6.1,0,11.19,4.38,12.28,10.17,2.14.56,3.72,2.51,3.72,4.83,0,.03,0,.07-.01.1,2.29.46,4.01,2.48,4.01,4.9Z" style="fill: none; stroke: #fff; stroke-width: 2px;"></path></g></svg>`;
}
