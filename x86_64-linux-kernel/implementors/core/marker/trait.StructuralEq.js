(function() {var implementors = {};
implementors["raw_cpuid"] = [{"text":"impl StructuralEq for CacheType","synthetic":false,"types":[]},{"text":"impl StructuralEq for TopologyType","synthetic":false,"types":[]},{"text":"impl StructuralEq for Hypervisor","synthetic":false,"types":[]},{"text":"impl StructuralEq for L2Associativity","synthetic":false,"types":[]}];
implementors["trapframe"] = [{"text":"impl StructuralEq for UserContext","synthetic":false,"types":[]},{"text":"impl StructuralEq for GeneralRegs","synthetic":false,"types":[]}];
implementors["x86_64"] = [{"text":"impl StructuralEq for VirtAddr","synthetic":false,"types":[]},{"text":"impl StructuralEq for PhysAddr","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;PortRead&gt; StructuralEq for PortReadOnly&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;PortWrite&gt; StructuralEq for PortWriteOnly&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;PortReadWrite&gt; StructuralEq for Port&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl StructuralEq for Cr0Flags","synthetic":false,"types":[]},{"text":"impl StructuralEq for Cr3Flags","synthetic":false,"types":[]},{"text":"impl StructuralEq for Cr4Flags","synthetic":false,"types":[]},{"text":"impl StructuralEq for EferFlags","synthetic":false,"types":[]},{"text":"impl StructuralEq for RFlags","synthetic":false,"types":[]},{"text":"impl StructuralEq for SegmentSelector","synthetic":false,"types":[]},{"text":"impl StructuralEq for DescriptorFlags","synthetic":false,"types":[]},{"text":"impl StructuralEq for PageFaultErrorCode","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for PhysFrame&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for PhysFrameRange&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for PhysFrameRangeInclusive&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl StructuralEq for Size4KiB","synthetic":false,"types":[]},{"text":"impl StructuralEq for Size2MiB","synthetic":false,"types":[]},{"text":"impl StructuralEq for Size1GiB","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for Page&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for PageRange&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;PageSize&gt; StructuralEq for PageRangeInclusive&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl StructuralEq for PageTableFlags","synthetic":false,"types":[]},{"text":"impl StructuralEq for PageTableIndex","synthetic":false,"types":[]},{"text":"impl StructuralEq for PageOffset","synthetic":false,"types":[]},{"text":"impl StructuralEq for PrivilegeLevel","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()